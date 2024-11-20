# Internal audit of near-governance-test
The review has been performed based on the contract code in the following repository:<br>
`https://github.com/valory-xyz/near-governance-test` <br>
commit: `dd3966e74acc5772716a28df6636ccee8da11d3b` or `0.1.0-pre-internal-audit`<br> 

## Objectives
The audit focused on contracts in this repo.

### Issue by BlockSec list
#### find Promises that are not handled
no
#### missing macro #[private] for callback functions
no
#### find functions that are vulnerable to reentrancy attack
no
#### lack of overflow check for arithmetic operation
no
#### missing check of sender != receiver
no
#### incorrect type used in parameters or return values
no
#### changes to collections are not saved
no
#### find nft_transfer without check of approval id
N/A
#### find approve or revoke functions without owner check
N/A
#### precision loss due to incorrect operation order
no
#### rounding without specifying ceil or floor
no
#### panic in callback function may lock contract
no
#### no assert_one_yocto in privileged function
no
#### duplicate id uses in collections
no
#### no panic on unregistered transfer receivers
N/A
#### find all unimplemented NEP interface
N/A
#### missing check of prepaid gas in ft_transfer_call
N/A
#### macro #[private] used in non-callback function
no
#### function result not used or checked
no
#### no upgrade function in contract
no
#### tautology used in conditional branch
no
#### missing balance check for storage expansion
no
#### missing balance check before storage unregister
N/A

### Medium issue
1. No function `change_hash` via governance vaa 

### Low issue
1. Exclude "-test" from name of project. Change to `governor_near` / or change Cargo.toml too. Fixing all to single name
2. Fixing README.md - `Build the code:` - incorrect
3. Fixing README.md - remove sandbox part as outdated  
4. Fixing setup-env.sh to actual versions
5. Ref state/byte_utils to Wormhole in README
6. Group all private functions in one place.

