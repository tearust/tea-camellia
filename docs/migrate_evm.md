# Key concepts
## Relationship between layer1 and layer2
### Responsibilities
Layer1 is blockchain. If on EVM, it is smart contracts
Layer2 is distributed cloud computing layer. 

Application runs on layer2. Application knows nothing about layer1. 

Layer1 has basic logic to validate layer2 nodes are hardware trusted.

Layer1 also has ERC20 ERC721 compatible interface.

### Communications between layers
Layer2 listen to layer1 blockchain events.
Layer2 state maintainer send layer1 txn.

For example, end user topup fund from layer1 to layer2. he send topup txn from front-end (browser metamask) to layer1. Layer1 smart contract lock the fund in a reserved account. Layer2 state maintainers listen to topup event, verify the fund has been locked, then mint layer2 TEA token to the same user layer2 account. 

Another exmple, end user wants to withdraw fund from layer2 to layer1. He posts withdraw request in layer2(Note this is a layer2 api call. We still call it a transation, but it is not a layer1 transaction). State maintainers receive this withdraw request, burn the minted TEA token(in layer2) then send a layer1 withdraw txn to layer1. Layer1 verify the multisig withdraw txn, then transfer the same amount of TEA from the reserved account back to this user's layer1 account.

This is similar to a standard cross chain bridge. But it is not cross chain, it is cross layers.
## State maintainer
Similar to PoS stakers. They maintain a state of layer2. 

Maintainers send layer1 txns to layer1.

Layer1 needs to verify more than half of total maintainer nodes signed in this txns before execute.

### State maintainer member change
#### Early state (version 1)
In epoch10 and likely the first milestone of mainnet, the member of state maintainers are added/removed by sudo. This is a temporary and centralized solution for early stage.

In the first milestone (early stage), change maintainer public key only verify sender is sudo.

# Smart contracts in the scope
There should be the following smart contracts

- TEA_ERC_20: This is used for TEA token.  ERC_20 contract
- CML_ERC_721: This is used for CML token. ERC_721 contract
- Lock: Topup and Withdraw between layer1 and layer2
- Maintainer: This is to maintain a list of active maintainer public key. They can modify a series of mapping data storage

# TEA_ERC_20
This is standard ERC20. 

## Genesis and Vesting

We can use https://github.com/abdelhamidbakhta/token-vesting-contracts as our vesting

We issue tokens to team, reserve, investors with a predefined vesting schedule.

# Lock

## Storage

hard coded reserved_address: This is a hard coded value, such as 0x0000000.... that we all know there is not a coresponding private key. Because it is hard coded constant value, no need to put it into storage.

When end user topup fund to layer2, the fund is actually locked in this reserved_address. When withdraw, the layer1 smart contract verify multisig from layer2 state maintainers then unlock the fund and transfer from reserved_address back to this user.

## Events
###  TAppTopup
```
		/// Fired after topuped successfully, event parameters:
		///  From account
		///  Topup amount
		///  Curent block number
		TAppTopup(T::AccountId, BalanceOf<T>, T::BlockNumber),
```

### TAppWithdraw
```
		/// Fired after topuped successfully, event parameters:
		///  To account
		///  Topup amount
		///  Curent block number
		///  Tsid
		TAppWithdraw(
			T::AccountId,
			BalanceOf<T>,
			T::BlockNumber,
			Vec<u8>,
		),
```

## Txns
### Topup

This is exactly a standard transfer operation. Just has a fixed to_address. (its value is reserved_address)

This txn is sent from front end by the end user.

Params:
- amount: how much to topup in TEA
- sender: address

Verify:
- sender signature
- fund balance

Action:
Just a typical transfer.

> Question: Do we still need this txn or event? Can we just use existing standard ERC20 transfer?

### Withdraw
This is transfer txn with special multisig verification operation. 

State maintainers agrees and signed to transfer fund from the mulsig account (reserved_account) back to the end user account.

Params:
- to_address: Address. The end user address
- combined_publickey_signature

Verify 
- More than half of maintainer signed
- sufficient fund in reserve

Action:
Transfer fund from reserved account to to_address

> Question: Why we cannot use standard ERC20 multisig? 
> Answer: In our futher version, we may want to automatically change the maintainer public keys list when layer2 replace maintainer. Not sure if the existing ERC 20 multisig can easily update public keys list

# CML_ERC_721
Differences
- Only **SUDO** can generate new CML seeds

> Question: How could our layer2 marketplace call ERC721 txn to transfer ownership of CML?

## Txns
> Question, should we still need this to be a standalone smart contract? Can we combine it to other smart contract? Is there any future upgrade benefit as standalone

# Maintainer Contract
## Storage
- maintainers: [maintainer_pub_key]. All maintainer public keys
- issuer: [issuer_address] . an array of issuers address. Issuer are those TPM manufacturors. They can generate new machine TEA_id
- machines: Map<tea_id, (type, cml_id, owner_pubkey)>
- tappstore_startup_node_ips: [(tappstore_startup_nodes_ip, cmd_id)]
- network_bootstrap_ips: [ip_address]
- tapps: Map<tapp_id, (app_name, latest_cid)>//This is the table of tapp and its detail, especially the front end IPFS CID

## Genesis
Add AWS Nitro as issuer_id 0

Add first batch of TAppStore hosts IP addresses

There is no need to verify aws_nitro_issuer signature.

## Events
### updated_maintainer_key
params
- sender public key
- [maintainer_key]
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

### BoostrapIpChanges

### TAppStoreHostChanges

### TAppChanged

## Txns

### update_maintainer_key
params
- [maintainer_key]

verify
- sender is **sudo**

action
- replace the [maintainer_keys]

### RegisterIssuer

Only **SUDO** can send this txn

Params:
- issuer_address: Account
- issuer_ca
Verify:
- sender is sudo
Action:


### RegisterMachine

Only used to register a TEA_id of a AWS Nitro machine.

Params
- tea_id: TEAID. The TPM unique ID
- owner: Address. Can be issuer address if a new machine has not been sold
- issuer_address (If AWS Nitro use the fake one for AWS)


Verify
- if issuer_address is 0x0000... Do not verify sender_address == issuer_address. anyone can call. No restrict on sudo
- if issuer_address is not 0x00000.... Verify sender_address == issuer_address, and sender signature
- tea_id cannot be existing key. 

Action
- Insert machine map
- Emit Layer2InfoBinded 

### ChangeTappStoreHost

When new TAppStore app host start hosting, the state maintainer will send this txn to layer1. 

This txn need 50% state maintainer signature to pass

Params:
- Combined_signature_maintainers:
- Add_address_list: [address].
- Remove_address_list: [address]

Verify:
- Multisig from more than half of maintainer public keys
Action:
- Deduplicated existing address
- Add
- Remove


### TransferMachine
Existing owner of a machine (TEA_ID) transfer to a new owner

Params
- to_address: Address. The new owner address
- owner signature
Verify
- sender is existing owner

Action
- Update the machine map owner_address

### UpsertTapp
Update (if exists) or insert new TApp item in tapp table

Params
- tapp_id
- name //usually an app name plus a version number as name
- cid // the latest IPFS CID to this app front end code

Verify
- Multisig from more than half of maintainer public keys

Action
- Upsert the tapps map



# Potential changes in layer2 (Not in scope)

## EVM signature ECDSA not ED25519

EVM is not supporting ED25519. So we will need EVM signature ECDSA. 
the same private key but differernt public key (address). A same node will have an public_key/address in DOT, but another public_key/address in EVM. 

## Combined state maintainer signature

when state maintainers send layer1 txn (to EVM), every state maintainer node gather signature from more than 50% other nodes, then combine all these signature together to layer1 (EVM). This is something under discussion.

## Change of state maintainer.

In epoch10, we still use a naive version. That only SUDO can send change maintainer public key txn. 

The txn only include a new merkel root. Layer1 can verify if a public included in this merkel root. If yes, it is one of the maintainer node.

In the early state (epoch10 to mainnet or may be longer), the change of maintainer node not likely happen frequently.
