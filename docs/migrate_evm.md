# Smart contracts
There should be the following smart contracts

- TEA_ERC_20: This is used for TEA token. A standard ERC_20 contract
- CML_ERC_721: This is used for CML token. A modified version of ERC_721 contract
- Maintianer: This is to maintain a list of active maintainer public key.
- Lock: Topup and withdraw between layer1 and layer2. Unlock needs combined signature for state maintainers.
- machine: TEA_ID and owner_id mapping. 

# Potential changes in layer2

## EVM signature. not ED25519

EVM is not supporting ED25519. So we will need EVM signature. 
the same private key but differernt public key (address). A same node will have an public_key/address in DOT, but another public_key/address in EVM. 

## Combined state maintainer signature

when state maintainers send layer1 txn (to EVM), every state maintainer node gather signature from more than 50% other nodes, then combine all these signature together to layer1 (EVM).

This may reduce the complexity of smart contract. also reduce gas cost. I still have question on this proposal. 

## Change of state maintainer.
Change state maintainer need to send txn to EVM. Store the list in a Merkle tree can reduce the gas cost. Whoever become a new seat owner need to pay for the cost.
# maintiner contract
## Data structure
Map<Seat, address>

## Genesis

doing nothing

## Events

### MaintainerAdded
### MaintainerRemoved
## Txns
### AddMaintainer

### RemoveMaintainer

## RPCs
GetMaintainerList


# Lock contract
## Genesis

## Events
###  TAppTopup
```
		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		TAppTopup(T::AccountId, T::AccountId, BalanceOf<T>, T::BlockNumber),
```

### TAppWithdraw
```
		/// Fired after topuped successfully, event parameters:
		/// 2. From account
		/// 3. To account
		/// 4. Topup amount
		/// 5. Curent block number
		/// 6. Tsid
		TAppWithdraw(
			T::AccountId,
			T::AccountId,
			BalanceOf<T>,
			T::BlockNumber,
			Vec<u8>,
		),
```

## Txns
### Topup

This is lock operation.

Sender's fund transfer to a locking address. This locking address is a multisig account that controlled by the state maintainer group. 

This txn is sent from front end by the end user.

### Withdraw
This is unlock operation. 

State maintainers agrees and signed to transfer fund from the mulsig account back to the end user account.

# Machine Contract
## Genesis
Add AWS Nitro

> Note: For AWS Nitro node, end users (miners) can register by themselves. Because layer2 RA can easily detect fake Nitro nodes. No need to verify from layer1 issuer's signature. As long as this node is marked as "nitro node"

## RPCs

### ListTappStoreStartupHodes

## Events
### MachineTransfered
```
		/// Params:
		/// 1. tea_id
		/// 2. from account
		/// 3. to account
		MachineTransfered(TeaPubKey, T::AccountId, T::AccountId),
```

### Layer2InfoBinded 
```

		/// Params:
		/// 1. tea_id
		/// 2. cml id
		/// 3. owner
		Layer2InfoBinded(TeaPubKey, CmlId, T::AccountId),

```


## Errors
```
	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// The account has been registered that can't be used to register again
		IssuerOwnerRegistered,
		/// The given issuer not exist
		IssuerNotExist,
		/// The given issuer owner is invalid
		InvalidIssuerOwner,
		/// The given machine id is already exist
		MachineAlreadyExist,
		/// The given machine id is not exist
		MachineNotExist,
		/// Machine owner is not valid
		InvalidMachineOwner,
		/// Length of given lists not the same
		BindingItemsLengthMismatch,
		ConnIdLengthToLong,
		IpAddressLengthToLong,
		StartupMachineBindingsLengthToLong,
		StartupTappBindingsLengthToLong,
		StartupOwnerIsNone,
	}
```

## Txns

### RegisterIssuer

### RegisterMachine

### RegisterNitroMachine

### AddTappStoreHost

### TransferMachine

### RegisterForLayer2 Should we rename to BindingToCml?




