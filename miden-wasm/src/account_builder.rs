#![allow(dead_code)]
use alloc::{
    collections::BTreeMap,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::Display;

use assembly::Assembler;
use miden_crypto::merkle::MerkleError;
use miden_objects::{
    accounts::{
        Account, AccountCode, AccountId, AccountStorage, AccountStorageType, AccountType, SlotItem,
        StorageMap,
    },
    assets::{Asset, AssetVault},
    crypto::dsa::rpo_falcon512::SecretKey,
    AccountError, AssetVaultError, Digest, Felt, Word, ZERO,
};
use rand::Rng;

// CONSTANTS
// ================================================================================================
pub const DEFAULT_ACCOUNT_CODE: &str = "
    export.::miden::contracts::wallets::basic::receive_asset
    export.::miden::contracts::wallets::basic::create_note
    export.::miden::contracts::wallets::basic::move_asset_to_note
    export.::miden::contracts::auth::basic::auth_tx_rpo_falcon512
";

// ACCOUNT BUILDER
// ================================================================================================
#[derive(Default, Debug, Clone)]
pub struct AccountStorageBuilder {
    items: Vec<SlotItem>,
    maps: BTreeMap<u8, StorageMap>,
}

/// Builder for an `AccountStorage`, the builder can be configured and used multiple times.
impl AccountStorageBuilder {
    pub fn new() -> Self {
        Self {
            items: vec![],
            maps: BTreeMap::new(),
        }
    }

    pub fn add_item(&mut self, item: SlotItem) -> &mut Self {
        self.items.push(item);
        self
    }

    pub fn add_items<I: IntoIterator<Item = SlotItem>>(&mut self, items: I) -> &mut Self {
        for item in items.into_iter() {
            self.add_item(item);
        }
        self
    }

    #[allow(dead_code)]
    pub fn add_map(&mut self, index: u8, map: StorageMap) -> &mut Self {
        self.maps.insert(index, map);
        self
    }

    pub fn build(&self) -> AccountStorage {
        AccountStorage::new(self.items.clone(), self.maps.clone()).unwrap()
    }
}

/// Builder for an `Account`, the builder allows for a fluent API to construct an account. Each
/// account needs a unique builder.
#[derive(Debug, Clone)]
pub struct AccountBuilder<T> {
    assets: Vec<Asset>,
    storage_builder: AccountStorageBuilder,
    code: String,
    nonce: Felt,
    account_id_builder: AccountIdBuilder<T>,
}

impl<T: Rng> AccountBuilder<T> {
    pub fn new(rng: T) -> Self {
        Self {
            assets: vec![],
            storage_builder: AccountStorageBuilder::new(),
            code: DEFAULT_ACCOUNT_CODE.to_string(),
            nonce: ZERO,
            account_id_builder: AccountIdBuilder::new(rng),
        }
    }

    pub fn add_asset(mut self, asset: Asset) -> Self {
        self.assets.push(asset);
        self
    }

    pub fn add_assets<I: IntoIterator<Item = Asset>>(mut self, assets: I) -> Self {
        for asset in assets.into_iter() {
            self.assets.push(asset);
        }
        self
    }

    pub fn add_storage_item(mut self, item: SlotItem) -> Self {
        self.storage_builder.add_item(item);
        self
    }

    pub fn add_storage_items<I: IntoIterator<Item = SlotItem>>(mut self, items: I) -> Self {
        self.storage_builder.add_items(items);
        self
    }

    pub fn code<C: AsRef<str>>(mut self, code: C) -> Self {
        self.code = code.as_ref().to_string();
        self
    }

    pub fn nonce(mut self, nonce: Felt) -> Self {
        self.nonce = nonce;
        self
    }

    pub fn account_type(mut self, account_type: AccountType) -> Self {
        self.account_id_builder.account_type(account_type);
        self
    }

    pub fn storage_type(mut self, storage_type: AccountStorageType) -> Self {
        self.account_id_builder.storage_type(storage_type);
        self
    }

    pub fn build(mut self, assembler: Assembler) -> Result<(Account, Word), AccountBuilderError> {
        let vault = AssetVault::new(&self.assets).map_err(AccountBuilderError::AssetVaultError)?;
        let storage = self.storage_builder.build();
        self.account_id_builder.code(&self.code);
        self.account_id_builder.storage_root(storage.root());
        let (account_id, seed) = self.account_id_builder.build(assembler.clone())?;
        let account_code = AccountCode::compile(&self.code, assembler)
            .map_err(AccountBuilderError::AccountError)?;

        let account = Account::from_parts(account_id, vault, storage, account_code, self.nonce);
        Ok((account, seed))
    }

    /// Build an account using the provided `seed`.
    pub fn build_with_seed(
        mut self,
        seed: Word,
        assembler: Assembler,
    ) -> Result<Account, AccountBuilderError> {
        let vault = AssetVault::new(&self.assets).map_err(AccountBuilderError::AssetVaultError)?;
        let storage = self.storage_builder.build();
        self.account_id_builder.code(&self.code);
        self.account_id_builder.storage_root(storage.root());
        let account_id = self.account_id_builder.with_seed(seed, assembler.clone())?;
        let account_code = AccountCode::compile(&self.code, assembler)
            .map_err(AccountBuilderError::AccountError)?;
        Ok(Account::from_parts(
            account_id,
            vault,
            storage,
            account_code,
            self.nonce,
        ))
    }

    /// Build an account using the provided `seed` and `storage`.
    ///
    /// The storage items added to this builder will added on top of `storage`.
    pub fn build_with_seed_and_storage(
        mut self,
        seed: Word,
        mut storage: AccountStorage,
        assembler: Assembler,
    ) -> Result<Account, AccountBuilderError> {
        let vault = AssetVault::new(&self.assets).map_err(AccountBuilderError::AssetVaultError)?;
        let inner_storage = self.storage_builder.build();

        for (key, value) in inner_storage.slots().leaves() {
            // Explicitly cast to `u64` to silence "type annotations needed" error.
            // Using `as u64` makes the intended type clear and avoids type inference issues.
            if key != AccountStorage::SLOT_LAYOUT_COMMITMENT_INDEX as u64 {
                // don't copy the reserved key
                storage
                    .set_item(key as u8, *value)
                    .map_err(AccountBuilderError::AccountError)?;
            }
        }

        self.account_id_builder.code(&self.code);
        self.account_id_builder.storage_root(storage.root());
        let account_id = self.account_id_builder.with_seed(seed, assembler.clone())?;
        let account_code = AccountCode::compile(&self.code, assembler)
            .map_err(AccountBuilderError::AccountError)?;
        Ok(Account::from_parts(
            account_id,
            vault,
            storage,
            account_code,
            self.nonce,
        ))
    }

    /// Build an account using the provided `seed` and `storage`.
    /// This method also returns the seed and secret key generated for the account based on the
    /// provided RNG.
    ///
    /// The storage items added to this builder will added on top of `storage`.
    pub fn build_with_auth(
        self,
        assembler: &Assembler,
        rng: &mut impl Rng,
    ) -> Result<(Account, Word, SecretKey), AccountBuilderError> {
        let sec_key = SecretKey::with_rng(rng);
        let pub_key: Word = sec_key.public_key().into();

        let storage_item = SlotItem::new_value(0, 0, pub_key);
        let (account, seed) = self
            .add_storage_item(storage_item)
            .build(assembler.clone())?;
        Ok((account, seed, sec_key))
    }
}

#[derive(Debug)]
pub enum AccountBuilderError {
    AccountError(AccountError),
    AssetVaultError(AssetVaultError),
    MerkleError(MerkleError),

    /// When the created [AccountId] doesn't match the builder's configured [AccountType].
    SeedAndAccountTypeMismatch,

    /// When the created [AccountId] doesn't match the builder's `on_chain` config.
    SeedAndOnChainMismatch,
}

impl Display for AccountBuilderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Builder for an `AccountId`, the builder can be configured and used multiple times.
#[derive(Debug, Clone)]
pub struct AccountIdBuilder<T> {
    account_type: AccountType,
    storage_type: AccountStorageType,
    code: String,
    storage_root: Digest,
    rng: T,
}

impl<T: Rng> AccountIdBuilder<T> {
    pub fn new(rng: T) -> Self {
        Self {
            account_type: AccountType::RegularAccountUpdatableCode,
            storage_type: AccountStorageType::OffChain,
            code: DEFAULT_ACCOUNT_CODE.to_string(),
            storage_root: Digest::default(),
            rng,
        }
    }

    pub fn account_type(&mut self, account_type: AccountType) -> &mut Self {
        self.account_type = account_type;
        self
    }

    pub fn storage_type(&mut self, storage_type: AccountStorageType) -> &mut Self {
        self.storage_type = storage_type;
        self
    }

    pub fn code<C: AsRef<str>>(&mut self, code: C) -> &mut Self {
        self.code = code.as_ref().to_string();
        self
    }

    pub fn storage_root(&mut self, storage_root: Digest) -> &mut Self {
        self.storage_root = storage_root;
        self
    }

    pub fn build(
        &mut self,
        assembler: Assembler,
    ) -> Result<(AccountId, Word), AccountBuilderError> {
        let (seed, code_commitment) = account_id_build_details(
            &mut self.rng,
            &self.code,
            self.account_type,
            self.storage_type,
            self.storage_root,
            assembler,
        )?;

        let account_id = AccountId::new(seed, code_commitment, self.storage_root)
            .map_err(AccountBuilderError::AccountError)?;

        Ok((account_id, seed))
    }

    pub fn with_seed(
        &mut self,
        seed: Word,
        assembler: Assembler,
    ) -> Result<AccountId, AccountBuilderError> {
        let code = AccountCode::compile(&self.code, assembler)
            .map_err(AccountBuilderError::AccountError)?;
        let code_commitment = code.commitment();

        let account_id = AccountId::new(seed, code_commitment, self.storage_root)
            .map_err(AccountBuilderError::AccountError)?;

        if account_id.account_type() != self.account_type {
            return Err(AccountBuilderError::SeedAndAccountTypeMismatch);
        }

        if account_id.storage_type() != self.storage_type {
            return Err(AccountBuilderError::SeedAndOnChainMismatch);
        }

        Ok(account_id)
    }
}

// UTILS
// ================================================================================================

/// Returns the account's seed and code commitment.
///
/// This compiles `code` and performs the proof-of-work to find a valid seed.
pub fn account_id_build_details<T: Rng>(
    rng: &mut T,
    code: &str,
    account_type: AccountType,
    storage_type: AccountStorageType,
    storage_root: Digest,
    assembler: Assembler,
) -> Result<(Word, Digest), AccountBuilderError> {
    let init_seed: [u8; 32] = rng.gen();
    let code = AccountCode::compile(code, assembler).map_err(AccountBuilderError::AccountError)?;
    let code_commitment = code.commitment();
    let seed = AccountId::get_account_seed(
        init_seed,
        account_type,
        storage_type,
        code_commitment,
        storage_root,
    )
    .map_err(AccountBuilderError::AccountError)?;

    Ok((seed, code_commitment))
}
