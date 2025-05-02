#![allow(non_snake_case)]
#![no_std]
use soroban_sdk::{contract, contracttype, contractimpl, log, Env, Symbol, String, symbol_short, Address, Vec, Map};

// Property status enum
#[contracttype]
#[derive(Clone)]
pub enum PropertyStatus {
    Active,
    UnderMaintenance,
    ForSale,
    Inactive
}

// Property structure to store property details
#[contracttype]
#[derive(Clone)]
pub struct Property {
    pub property_id: u64,
    pub owner: Address,
    pub title: String,
    pub description: String,
    pub location: String,
    pub total_shares: u64,
    pub available_shares: u64,
    pub price_per_share: u64,
    pub status: PropertyStatus,
    pub registration_time: u64
}

// Share ownership record
#[contracttype]
#[derive(Clone)]
pub struct ShareOwnership {
    pub owner: Address,
    pub property_id: u64,
    pub shares: u64,
    pub acquisition_time: u64
}

// Mapping property_id to Property struct
#[contracttype]
pub enum PropertyRegistry {
    Property(u64)
}

// Mapping (property_id, owner) to ShareOwnership
#[contracttype]
pub enum OwnershipRegistry {
    Ownership(u64, Address)
}

// Counter for property IDs
const PROPERTY_COUNT: Symbol = symbol_short!("P_COUNT");

#[contract]
pub struct TokenizedPropertyContract;

#[contractimpl]
impl TokenizedPropertyContract {
    // Register a new property
    pub fn register_property(
        env: Env,
        owner: Address,
        title: String,
        description: String,
        location: String,
        total_shares: u64,
        price_per_share: u64
    ) -> u64 {
        // Authenticate the owner
        owner.require_auth();
        
        // Check valid property parameters
        if total_shares == 0 {
            log!(&env, "Total shares must be greater than zero");
            panic!("Total shares must be greater than zero");
        }
        
        // Generate new property ID
        let mut property_count: u64 = env.storage().instance().get(&PROPERTY_COUNT).unwrap_or(0);
        property_count += 1;
        
        // Get current timestamp
        let registration_time = env.ledger().timestamp();
        
        // Create new property
        let property = Property {
            property_id: property_count,
            owner: owner.clone(),
            title,
            description,
            location,
            total_shares,
            available_shares: total_shares, // Initially all shares are available
            price_per_share,
            status: PropertyStatus::Active,
            registration_time
        };
        
        // Store property data
        env.storage().instance().set(&PropertyRegistry::Property(property_count), &property);
        
        // Create initial ownership record for the property owner
        let ownership = ShareOwnership {
            owner: owner.clone(),
            property_id: property_count,
            shares: 0, // Initially the owner has no purchased shares
            acquisition_time: registration_time
        };
        
        // Store ownership data
        env.storage().instance().set(
            &OwnershipRegistry::Ownership(property_count, owner.clone()),
            &ownership
        );
        
        // Update property count
        env.storage().instance().set(&PROPERTY_COUNT, &property_count);
        
        // Extend TTL for data persistence
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Property registered with ID: {}", property_count);
        property_count
    }
    
    // Purchase shares of a property
    pub fn purchase_shares(
        env: Env,
        buyer: Address,
        property_id: u64,
        shares_to_buy: u64
    ) -> u64 {
        // Authenticate the buyer
        buyer.require_auth();
        
        // Get property data
        let key = PropertyRegistry::Property(property_id);
        let mut property: Property = env.storage().instance().get(&key)
            .expect("Property not found");
            
        // Check if property is active and for sale
        if !matches!(property.status, PropertyStatus::Active | PropertyStatus::ForSale) {
            log!(&env, "Property is not available for purchase");
            panic!("Property is not available for purchase");
        }
        
        // Check if enough shares are available
        if shares_to_buy > property.available_shares {
            log!(&env, "Not enough shares available");
            panic!("Not enough shares available");
        }
        
        // Get current timestamp
        let acquisition_time = env.ledger().timestamp();
        
        // Update available shares
        property.available_shares -= shares_to_buy;
        
        // Get or create ownership record for the buyer
        let ownership_key = OwnershipRegistry::Ownership(property_id, buyer.clone());
        let mut ownership: ShareOwnership = env.storage().instance().get(&ownership_key).unwrap_or(
            ShareOwnership {
                owner: buyer.clone(),
                property_id,
                shares: 0,
                acquisition_time
            }
        );
        
        // Update ownership record
        ownership.shares += shares_to_buy;
        
        // Store updated data
        env.storage().instance().set(&key, &property);
        env.storage().instance().set(&ownership_key, &ownership);
        
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Shares purchased for property: {}", property_id);
        property.price_per_share * shares_to_buy // Return total price
    }
    
    // Update property status (owner only)
    pub fn update_property_status(
        env: Env,
        owner: Address,
        property_id: u64,
        new_status: PropertyStatus
    ) {
        // Authenticate the owner
        owner.require_auth();
        
        // Get property data
        let key = PropertyRegistry::Property(property_id);
        let mut property: Property = env.storage().instance().get(&key)
            .expect("Property not found");
            
        // Verify the owner
        if property.owner != owner {
            log!(&env, "Only the owner can update property status");
            panic!("Only the owner can update property status");
        }
        
        // Update property status
        property.status = new_status;
        
        // Store updated property data
        env.storage().instance().set(&key, &property);
        
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Property status updated: {}", property_id);
    }
    
    // Update property price per share (owner only)
    pub fn update_price(
        env: Env,
        owner: Address,
        property_id: u64,
        new_price: u64
    ) {
        // Authenticate the owner
        owner.require_auth();
        
        // Get property data
        let key = PropertyRegistry::Property(property_id);
        let mut property: Property = env.storage().instance().get(&key)
            .expect("Property not found");
            
        // Verify the owner
        if property.owner != owner {
            log!(&env, "Only the owner can update property price");
            panic!("Only the owner can update property price");
        }
        
        // Update property price
        property.price_per_share = new_price;
        
        // Store updated property data
        env.storage().instance().set(&key, &property);
        
        env.storage().instance().extend_ttl(10000, 10000);
        
        log!(&env, "Property price updated: {}", property_id);
    }
    
    // Get property details
    pub fn get_property(env: Env, property_id: u64) -> Property {
        let key = PropertyRegistry::Property(property_id);
        env.storage().instance().get(&key).expect("Property not found")
    }
    
    // Get ownership details for a specific owner and property
    pub fn get_ownership(env: Env, property_id: u64, owner: Address) -> ShareOwnership {
        let key = OwnershipRegistry::Ownership(property_id, owner.clone());
        env.storage().instance().get(&key).unwrap_or(
            ShareOwnership {
                owner: owner.clone(),
                property_id,
                shares: 0,
                acquisition_time: 0
            }
        )
    }
}