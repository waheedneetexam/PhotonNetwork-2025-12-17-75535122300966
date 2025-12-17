use ic_cdk::api::caller;
use ic_cdk::api::management_canister::ecdsa::{
    ecdsa_public_key, EcdsaCurve, EcdsaKeyId, EcdsaPublicKeyArgument,
};
use ic_cdk::api::management_canister::bitcoin::{
    bitcoin_get_utxos, 
    bitcoin_get_current_fee_percentiles, // <--- FIXED: Added missing import
    BitcoinNetwork as IcpBitcoinNetwork, 
    GetUtxosRequest,
    GetCurrentFeePercentilesRequest // <--- FIXED: Added missing struct
};
use bitcoin::{Address, Network, PublicKey};

// --- HELPER 1: KEY ID CONFIG ---
fn get_key_id() -> EcdsaKeyId {
    EcdsaKeyId {
        curve: EcdsaCurve::Secp256k1,
        // Use "test_key_1" for Playground/Testnet
        name: "test_key_1".to_string(), 
    }
}

// --- HELPER 2: NETWORK CONFIG ---

// 1. For Address Generation (crate: bitcoin)
fn get_network() -> Network {
    Network::Testnet 
}

// 2. For API Calls (crate: ic_cdk)
fn get_icp_network() -> IcpBitcoinNetwork {
    IcpBitcoinNetwork::Testnet 
}

// --- CORE FUNCTION 1: GET ADDRESS ---
#[ic_cdk::update]
async fn get_btc_address() -> String {
    let user_principal = caller();

    // 1. Get ECDSA Public Key
    let (response,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: vec![user_principal.as_slice().to_vec()],
        key_id: get_key_id(),
    })
    .await
    .expect("Failed to fetch public key");

    // 2. Parse the Public Key
    let public_key = PublicKey::from_slice(&response.public_key)
        .expect("Invalid public key from ICP");

    // 3. Generate the Address (Uses get_network)
    let address = Address::p2wpkh(&public_key, get_network())
        .expect("Failed to create address");

    // 4. Return as String
    address.to_string()
}

// --- CORE FUNCTION 2: GET MY BALANCE ---
#[ic_cdk::update]
async fn get_my_balance() -> String {
    let user_principal = caller();

    // 1. Re-derive the address
    let (pk_response,) = ecdsa_public_key(EcdsaPublicKeyArgument {
        canister_id: None,
        derivation_path: vec![user_principal.as_slice().to_vec()],
        key_id: get_key_id(),
    }).await.expect("Failed to fetch public key");

    let public_key = PublicKey::from_slice(&pk_response.public_key)
        .expect("Invalid public key");
        
    // Uses get_network() for Address creation
    let my_address_str = Address::p2wpkh(&public_key, get_network())
        .expect("Failed to create address")
        .to_string();
    
    // 2. Ask Bitcoin Network
    // FIX: Must use get_icp_network() here, NOT get_network()
    let (response,) = bitcoin_get_utxos(GetUtxosRequest {
        network: get_icp_network(),  // <--- FIXED
        address: my_address_str.clone(),
        filter: None, 
    })
    .await
    .expect("Failed to fetch UTXOs");

    // 3. Sum Balance
    let mut total_sats = 0;
    for utxo in response.utxos {
        total_sats += utxo.value;
    }

    format!("Checked Address: {} | Balance: {}", my_address_str, total_sats)
}

// --- DEBUG FUNCTION: CHECK ANY ADDRESS ---
#[ic_cdk::update]
async fn check_address_balance(address: String) -> String {
    // FIX: Must use get_icp_network() here, NOT get_network()
    let (response,) = bitcoin_get_utxos(GetUtxosRequest {
        network: get_icp_network(), // <--- FIXED
        address: address.clone(), 
        filter: None, 
    })
    .await
    .expect("Failed to fetch UTXOs");

    let mut total_sats = 0;
    for utxo in response.utxos {
        total_sats += utxo.value;
    }

    format!("Address: {} | Balance: {}", address, total_sats)
}

// --- DEBUG FUNCTION: CHECK CONNECTION STATUS ---
#[ic_cdk::update]
async fn debug_network_status() -> String {
    // FIX: Uses get_icp_network()
    let fees_result = bitcoin_get_current_fee_percentiles(
        GetCurrentFeePercentilesRequest {
            network: get_icp_network(), // <--- FIXED
        }
    ).await;

    match fees_result {
        Ok(_) => "ICP Connection: ONLINE. (If balance is 0, adapter is lagging)".to_string(),
        Err(e) => format!("ICP Connection: OFFLINE. Error: {:?}", e),
    }
}

ic_cdk::export_candid!();