contract;

use std::storage::StorageMap;

storage {
    /// Total token supply
    total_supply: u64,
    /// Token balances for each address
    balances: StorageMap<Address, u64>,
    /// Name of the token
    name: str[32],
    /// Symbol of the token
    symbol: str[10],
    /// Number of decimal places
    decimals: u8,
    /// Approved amounts for spenders
    allowances: StorageMap<(Address, Address), u64>,
}

abi Token {
    // Standard token info
    #[storage(read)]
    fn name() -> str[32];

    #[storage(read)]
    fn symbol() -> str[10];

    #[storage(read)]
    fn decimals() -> u8;

    #[storage(read)]
    fn total_supply() -> u64;

    // Balance checking
    #[storage(read)]
    fn balance_of(address: Address) -> u64;

    // Allowance checking
    #[storage(read)]
    fn allowance(owner: Address, spender: Address) -> u64;

    // Transaction details
    #[storage(read)]
    fn get_transaction_details(tx_id: b256) -> TransactionInfo;
}

// Struktury do przechowywania szczegółowych informacji
struct TransactionInfo {
    // Basic info
    sender: Address,
    recipient: Address,
    amount: u64,
    timestamp: u64,

    // Additional details
    transaction_type: TransactionType,
    status: TransactionStatus,
    gas_used: u64,
    block_number: u64,
}

enum TransactionType {
    Transfer: (),
    Mint: (),
    Burn: (),
    Approve: (),
}

enum TransactionStatus {
    Success: (),
    Failed: (),
    Pending: (),
}

impl Token for Contract {
    #[storage(read)]
    fn name() -> str[32] {
        storage.name
    }

    #[storage(read)]
    fn symbol() -> str[10] {
        storage.symbol
    }

    #[storage(read)]
    fn decimals() -> u8 {
        storage.decimals
    }

    #[storage(read)]
    fn total_supply() -> u64 {
        storage.total_supply
    }

    #[storage(read)]
    fn balance_of(address: Address) -> u64 {
        storage.balances.get(address).unwrap_or(0)
    }

    #[storage(read)]
    fn allowance(owner: Address, spender: Address) -> u64 {
        storage.allowances.get((owner, spender)).unwrap_or(0)
    }

    #[storage(read)]
    fn get_transaction_details(tx_id: b256) -> TransactionInfo {
        // Tu zaimplementuj logikę pobierania szczegółów transakcji
        // W rzeczywistej implementacji będziesz musiał śledzić te informacje
        // w osobnej mapie storage lub pobierać je z historii bloków
        TransactionInfo {
            sender: Address::from(0x0000000000000000000000000000000000000000),
            recipient: Address::from(0x0000000000000000000000000000000000000000),
            amount: 0,
            timestamp: 0,
            transaction_type: TransactionType::Transfer,
            status: TransactionStatus::Success,
            gas_used: 0,
            block_number: 0,
        }
    }
}