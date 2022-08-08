# Key concepts
## Relationship between layer1 and layer2
### Responsibilities
Layer1 is blockchain. If on EVM, it is smart contracts
Layer2 is distributed cloud computing layer. 

Application runs on layer2. 

Layer1 has basic logic to validate layer2 nodes are hardware trusted.

Layer1 also has ERC20 ERC721 compatible interface.

### Communications between layers
Layer2 listen to layer1 blockchain events.
Layer2 state maintainer send layer1 txn.

For example, end user topup fund from layer1 to layer2. he send topup txn on layer1. Layer1 smart contract lock the fund in a reserved account. Layer2 state maintainers listen to topup event, verify the fund has been locked, then mint layer2 TEA token to this user layer2 account.

Another exmple, end user withdraw fund from layer2 to layer1. He send withdraw txn in layer2. State maintainers burn the minted TEA token then send a withdraw txn to layer1 that transfer the same amount of TEA from the reserved account back to this user's layer1 account.

## State maintainer
Similar to PoS stakers. They maintain a state of layer2. 

Layer1 can verify if a public key is one of the layer2 state maintainer. Layer1 also knows the total number of layer2 state maintainer nodes. Some txns sent to layer1 need to be verified to be signed by greater than 50% of layer2 maintainer nodes.

### State maintainer member change
#### Early state (version 1)
In epoch10 and likely the first milestone of mainnet, the member of state maintainers are added/removed by sudo. This is a temporary and centralized solution for early stage.

There is a concept called **Seat**. It is a concept related to profit only. In the early state, a seat owner only share revenue and pay tax. He cannot setup a state maintainer nodes and add its public key to the group. This will be allowed a later milestone.

In the first milestone (early stage), change maintainer public key only verify sender is sudo.

#### Later stage (not sure the version yet)
We do not need to implement this for now. it is just a design. NOT IN THE SCOPE!!!

In the future official version, a seat owner needs to add its own state maintainer node public key to the merkel root. So the merkel root will change everytime the seat ownership change.

# Smart contracts
There should be the following smart contracts

- TEA_ERC_20: This is used for TEA token. A modified ERC_20 contract
- CML_ERC_721: This is used for CML token. A modified version of ERC_721 contract
- Maintainer: This is to maintain a list of active maintainer public key.
- Lock: Topup and withdraw between layer1 and layer2. Unlock needs combined signature for state maintainers.
- machine: TEA_ID and owner_id mapping. 

# TEA_ERC_20
This is modified ERC20. We only list the differences.

## Storage

hard coded reserved_address: This is a hard coded value, such as 0x0000000 that we all know there is known coresponding private key. Because it is hard coded constant value, no need to put it into storage.

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

Sender's fund transfer to a reserved (or called locking) address. This locking address is a multisig account that controlled by the state maintainers group. 

This txn is sent from front end by the end user.

Params:
- amount: how much to topup in TEA

Verify:
- sender signature
- fund balance
Action:
It is just a simpple transfer txn. 
The transfer_to account is internal reserved_address


### Withdraw
This is unlock operation. 

State maintainers agrees and signed to transfer fund from the mulsig account back to the end user account.

Params:
- to_address
- combined_publickey_signature

Verify 
- This txn is not send during layer1 delayed grace period.
- sender public key is included in the maintainer merkel_root. this makes sure the sender is one of the state maintainer. **Need to lookup State Maintainer contract**
- combined_publickey_signature 1)public key is included in the maintainer merkel_root 2)signature for each public key 3)the number of public key exceed 50% of maintainer nodes number. 
- sufficient fund in reserve

Action:
Transfer fund from reserved account to to_address

# CML_ERC_721
Differences
- Only **SUDO** can generate new CML seeds

# maintainer contract (early stage version)
## Storage
Merkel_root: Bytes,Verify if a public key included in the maintainer nodes group.
total_maintainer: u32, How many total active maintainer nodes.
## Genesis
Merkel_root empty.
total_maintainer zero.

## RPC
### if_public_key_included

input
- public key
output
- bool
logic: if the public key is included in the merkel_root, returns true, otherwise false

### get_total_maintainer
output
- u32, total_maitainer

## Events
### merkel_root_changed
params
- sender public key
- current merkel_root
### total_maintainer_changed
params
- sender public key
- current total_maintainer

## Txns
### update_merkel_root
params
- new_merkel_root

verify
- sender is **sudo**

action
- replace merkel_root

> Question, should we still need this to be a standalone smart contract? For future update?

# Maintainer contract (later stage)
TODO

# Machine Contract
## Storage
issuer: List<issuer_address>
machine:Map<tea_id, (type, cml_id, owner_pubkey)>
tappstore_startup_node_ips: List<tappstore_startup_nodes_ip>
network_bootstrap_ips: List<ip_address>

## Genesis
Add AWS Nitro as issuer_id 0

Add first batch of TAppStore hosts IP addresses

There is no need to verify aws_nitro_issuer signature.
## RPCs

### ListTappStoreStartupHodes
params
- None
return:
- List of issuer public keys

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

## Txns

### RegisterIssuer

Only **SUDO** can send this txn

Params:
- issuer_address: Account
Verify:
- sender is sudo
Action:
- create new seq id (existing issuer_seq_id ++)


### RegisterMachine

Only used to register a TEA_id of a AWS Nitro machine.

Params
- tea_id: TEAID. The TPM unique ID
- owner: Address. Can be issuer address if a new machine has not been sold
- issuer_address (If AWS Nitro use the fake one for AWS)
- issuer_signature: bytes
- owner_signature: Bytes


Verify
- anyone can call. No restrict on sudo
- Verify issuer signature, No need to verify issuer signature for AWS_NITRO
- Verify owner signature. owner is the txn sender

Action
- Insert machine map
- Emit Layer2InfoBinded 

### AddTappStoreHost

When new TAppStore app host start hosting, the state maintainer will send this txn to layer1. 

This txn need 50% state maintainer signature to pass


### TransferMachine
Existing owner of a machine (TEA_ID) transfer to a new owner

Params
- to_address: Address. The new owner address
- owner signature
Verify
- sender is existing owner

Action
Update the machine map owner_address

### RegisterForLayer2 Should we rename to BindingToCml?
When miner plant a CML. The TEA_ID will be bundled with this CML id
Sender is the owner of the CML
Params
- cml_id

Verify
- sender is the CML owner
- sender is the machine owner

Action
- Update machines map. cml_id

# Potential changes in layer2

## EVM signature ECDSA not ED25519

EVM is not supporting ED25519. So we will need EVM signature ECDSA. 
the same private key but differernt public key (address). A same node will have an public_key/address in DOT, but another public_key/address in EVM. 

## Combined state maintainer signature

when state maintainers send layer1 txn (to EVM), every state maintainer node gather signature from more than 50% other nodes, then combine all these signature together to layer1 (EVM). This is something under discussion.

## Change of state maintainer.

In epoch10, we still use a naive version. That only SUDO can send change maintainer public key txn. 

The txn only include a new merkel root. Layer1 can verify if a public included in this merkel root. If yes, it is one of the maintainer node.

In the early state (epoch10 to mainnet or may be longer), the change of maintainer node not likely happen frequently.
