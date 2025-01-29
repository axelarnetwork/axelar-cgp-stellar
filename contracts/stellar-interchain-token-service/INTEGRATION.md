# ITS Integration

## ITS Hub

The Stellar implementation of the Interchain Token Service (ITS) communicates with other ITS contracts through the [ITS Hub](https://docs.axelar.dev/dev/amplifier/its-hub/introduction/), a Cosmwasm smart contract deployed on the Axelar network which acts as central routing hub for interchain token transfers. All interchain token functionality, such as transfering tokens or deploying Interchain Token contracts is achieved via messages sent to or received from the ITS Hub. Direct communication between Stellar ITS and ITS contracts on other chains is not supported.

## Differences between Stellar ITS and EVM ITS

- Token contract deployment is handled directly by the ITS contract, rather than through a factory contract.
- Stellar ITS communicates with ITS edge contracts via the ITS Hub, so there is no need to store addresses of trusted ITS contracts on other chains. Instead, Stellar ITS stores a list of trusted chains with which it can interact throught the ITS Hub.
- Flow limit is enforced by the ITS contract directly rather than by individual Token Managers.
- Transfer functionality (mint/burn or lock/unlock of tokens) is handled directly by the ITS contract rather than Token Handler contracts.
