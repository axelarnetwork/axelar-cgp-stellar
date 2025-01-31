# Interchain Token Service Integration

## ITS Hub

The Stellar implementation of the Interchain Token Service (ITS) communicates with other ITS contracts through the [ITS Hub](https://docs.axelar.dev/dev/amplifier/its-hub/introduction/), a Cosmwasm smart contract deployed on the Axelar network which acts as central routing hub for interchain token transfers. All interchain token functionality, such as transfering tokens or deploying Interchain Token contracts is achieved via messages sent to or received from the ITS Hub. Direct communication between Stellar ITS and ITS contracts on other chains is not supported.

## Differences between Stellar ITS and EVM ITS

### Contract size restrictions

Stellar allows significantly larger contract code size than EVM. This allows some functionality to be contained within the Stellar ITS contract without the need for splitting into smaller contracts:

- Stellar ITS has no Factory contract. Functionality such as deploying token contracts, registering canonical tokens, and computing token IDs and deployment salts is handled directly by the ITS contract.
- Flow limit is tracked and enforced by the ITS contract directly rather than by individual Token Managers.

### Authorization

Soroban's authorization model differs slightly from EVM, as there is no notion of a `msg.sender`. If a function requires authorizing the caller, the calling address must be passed as a parameter.

### Trusted chain

Stellar ITS communicates with ITS edge contracts via the ITS Hub, so there is no need to store addresses of trusted ITS edge contracts deployed on other chains. Instead, Stellar ITS stores a list of trusted chains with which it can interact through the ITS Hub.

### TTL

Soroban contract code and storage has a Time To Live (TTL), after which it is either archived or deleted (depending on the storage type). TTL must be extended periodically in order to keep the data live. TTL extensions are handled within the Stellar ITS contract logic.

### Native currency

Soroban smart contracts interact with the Stellar native currency (XLM) through a Stellar Asset Contract (SAC). The address of the XLM SAC is written to storage in the Stellar ITS constructor.

### Data serialization

Stellar ITS provides a module for ABI encoding/decoding GMP data for transmitting and receiving from the ITS Hub.

### Unsupported features

The following features are not supported by Stellar ITS at this time:

- Linking custom tokens
- `MintBurnFrom`, `LockUnlockFee`, and `MintBurn` Token Manager types
