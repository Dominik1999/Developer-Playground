#![no_std]
extern crate alloc;

mod account_builder;
mod mock_chain;
mod mock_host;
mod transaction_context;
mod transaction_context_builder;

use crate::transaction_context_builder::TransactionContextBuilder;

use alloc::{
    collections::BTreeMap,
    format,
    rc::Rc,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use rand_chacha::{rand_core::SeedableRng, ChaCha20Rng};

use miden_lib::transaction::TransactionKernel;
use miden_objects::{
    accounts::{Account, AccountCode, AccountId, AccountStorage, AuthSecretKey, SlotItem},
    assets::{Asset, AssetVault, FungibleAsset},
    crypto::dsa::rpo_falcon512::SecretKey,
    notes::{
        Note, NoteAssets, NoteExecutionHint, NoteInputs, NoteMetadata, NoteRecipient, NoteScript,
        NoteType,
    },
    transaction::{TransactionArgs, TransactionScript},
    Felt, NoteError, Word, ZERO, AccountError, TransactionScriptError,
};
use miden_tx::{auth::BasicAuthenticator, TransactionExecutor};
use wasm_bindgen::prelude::*;

// CONSTANTS
// ================================================================================================

pub const ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN: u64 = 0x200000000000001f; // 2305843009213693983
pub const ACCOUNT_ID_SENDER: u64 = 0x800000000000001f; // 9223372036854775839
pub const ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN: u64 = 0x900000000000003f; // 10376293541461622847

#[wasm_bindgen]
pub fn example(account_code: &str, note_script: &str, note_inputs: Option<Vec<u64>>, transaction_script: &str) -> JsValue {
    match inner_example(account_code, note_script, note_inputs, transaction_script) {
        Ok(result) => JsValue::from_str(&result),
        Err(err) => JsValue::from_str(&format!("Error: {:?}", err)),
    }
}

pub fn inner_example(account_code: &str, note_script: &str, note_inputs: Option<Vec<u64>>, transaction_script: &str) -> Result<String, JsValue> {
    // Validate input scripts
    if account_code.is_empty() || note_script.is_empty() || note_inputs.is_none() || transaction_script.is_empty() {
        return Err(JsValue::from_str("Input cannot be empty"));
    }

    // Create assets
    let faucet_id = AccountId::try_from(ACCOUNT_ID_FUNGIBLE_FAUCET_ON_CHAIN).unwrap();
    let fungible_asset: Asset = FungibleAsset::new(faucet_id, 100).unwrap().into();

    // Create sender and target account
    let sender_account_id = AccountId::try_from(ACCOUNT_ID_SENDER).unwrap();

    // CONSTRUCT USER ACCOUNT
    // --------------------------------------------------------------------------------------------
    let target_account_id =
        AccountId::try_from(ACCOUNT_ID_REGULAR_ACCOUNT_UPDATABLE_CODE_OFF_CHAIN).unwrap();
    let (target_pub_key, falcon_auth) = get_new_pk_and_authenticator();

    let target_account =
        get_account_with_account_code(account_code, target_account_id, target_pub_key, None).map_err(|err| JsValue::from_str(&err.to_string()))?;

    // CONSTRUCT NOTE
    // --------------------------------------------------------------------------------------------
    let note = get_note_with_fungible_asset_and_script(
        fungible_asset,
        note_script,
        sender_account_id,
        note_inputs.unwrap().iter().map(|&x| Felt::new(x)).collect(),
    )
    .map_err(|err| JsValue::from_str(&err.to_string()))?;

    // CONSTRUCT TX ARGS
    // --------------------------------------------------------------------------------------------
    let tx_script = build_transaction_script(transaction_script).map_err(|err| JsValue::from_str(&err.to_string()))?;
    let tx_args_target = TransactionArgs::with_tx_script(tx_script);

    // CONSTRUCT AND EXECUTE TX
    // --------------------------------------------------------------------------------------------
    let tx_context = TransactionContextBuilder::new(target_account.clone())
        .input_notes(vec![note.clone()])
        .build();

    let executor = TransactionExecutor::new(tx_context.clone(), Some(falcon_auth.clone()));

    let block_ref = tx_context.tx_inputs().block_header().block_num();
    let note_ids = tx_context
        .tx_inputs()
        .input_notes()
        .iter()
        .map(|note| note.id())
        .collect::<Vec<_>>();

    // Execute the transaction and get the witness
    let executed_transaction = executor
        .execute_transaction(target_account_id, block_ref, &note_ids, tx_args_target)
        .map_err(|err| JsValue::from_str(&err.to_string()))?;

    // Prove, serialize/deserialize and verify the transaction
    // assert!(prove_and_verify_transaction(executed_transaction.clone()).is_ok());

    Ok(format!("args: {:?}", executed_transaction.account_delta()))
}

pub fn get_account_with_account_code(
    account_code_src: &str,
    account_id: AccountId,
    public_key: Word,
    assets: Option<Asset>,
) -> Result<Account, AccountError> {
    let assembler = TransactionKernel::assembler().with_debug_mode(true);

    let account_code = AccountCode::compile(account_code_src, assembler).map_err(|err| err.into())?;
    let account_storage =
        AccountStorage::new(vec![SlotItem::new_value(0, 0, public_key)], BTreeMap::new()).unwrap();

    let account_vault = match assets {
        Some(asset) => AssetVault::new(&[asset]).unwrap(),
        None => AssetVault::new(&[]).unwrap(),
    };

    Ok(Account::from_parts(
        account_id,
        account_vault,
        account_storage,
        account_code,
        Felt::new(1),
    ))
}

pub fn get_note_with_fungible_asset_and_script(
    fungible_asset: Asset,
    note_script: &str,
    sender_id: AccountId,
    inputs: Vec<Felt>,
) -> Result<Note, NoteError> {
    let assembler = TransactionKernel::assembler().with_debug_mode(true);
    let note_script = NoteScript::compile(note_script, assembler).map_err(|err| err.into())?;
    const SERIAL_NUM: Word = [Felt::new(1), Felt::new(2), Felt::new(3), Felt::new(4)];

    let vault = NoteAssets::new(vec![fungible_asset.into()]).unwrap();
    let metadata = NoteMetadata::new(
        sender_id,
        NoteType::Public,
        1.into(),
        NoteExecutionHint::Always,
        ZERO,
    )
    .unwrap();
    let note_inputs = NoteInputs::new(inputs).unwrap();
    let recipient = NoteRecipient::new(SERIAL_NUM, note_script, note_inputs);

    Ok(Note::new(vault, metadata, recipient))
}

pub fn build_transaction_script(transaction_script: &str) -> Result<TransactionScript, TransactionScriptError> {
    let compiled_tx_script = TransactionScript::compile(transaction_script, [], TransactionKernel::assembler())
        .map_err(|err| err.into())?;
    Ok(compiled_tx_script)
}

pub fn get_new_pk_and_authenticator() -> (Word, Rc<BasicAuthenticator<ChaCha20Rng>>) {
    let seed = [0_u8; 32];
    let mut rng = ChaCha20Rng::from_seed(seed);

    let sec_key = SecretKey::with_rng(&mut rng);
    let pub_key: Word = sec_key.public_key().into();

    let authenticator = BasicAuthenticator::<ChaCha20Rng>::new_with_rng(
        &[(pub_key, AuthSecretKey::RpoFalcon512(sec_key))],
        rng,
    );

    (pub_key, Rc::new(authenticator))
}
