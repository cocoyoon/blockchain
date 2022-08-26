# Asset Tracking System by Substrate

## Shipment Tracking and Verification on Substrate

The aim of this project is to build a shipment tracking and verification blockchain using substrate. The Minimum Viable Product (MVP) will have the functionality to create a shipment with a unique ID and a predefined static route. The verification component will consist of generating a secret key for the next transit point of the shipment at every transit point.

### Example

Let there be:

- N transit points { TP1, TP2, ..., TPN}
- A shipment with:
  - route = { TP4, TP8, TP11 }
  - uid = 112
  - key = X
  - owner = TP8
- Package with a physical tag and seal that can be scanned to get the key

When TP8 receives the package, the following actions will be carried out:

1. TP8 calls an update function that accepts the package uid and scanned key
2. TP8 verifies that the key and the owner information is correct
3. TP8 generates a new key and update the owner of the package to TP11

When TP11 receives the package, the following actions will be carried out:

1. TP11 calls an update function that accepts the package uid and scanned key
2. TP11 verifies that the key and the owner information is correct
3. TP11 marks the package as shipped and set keys/owner to None

## Blockchain Overview

### Entities

1. Manager: One account that manages a number of transit points and has the power to add or remove transit points. _Alternatives to a centralized entity?_
2. Transit Points: Each transit point has a unique id and serves as both a sender and receiver of packages. In the MVP, each originating transit point also sets the route and calls a blockchain function that creates a key for the next transit point. This key is used to generate a machine readable code at the transit points and this code is embedded in the physical package. At the reciving transit point and the machine readable code is read to reveal the key. The combination of correct key and the correct owner is used to verify the integrity of the package and that of the supply chain.
3. Customers: In the MVP, customers can only access the state of the chain for the current location and status of the package.

## Technical Design Todo

- Implement algorithm to choose route
- See how we can make use of off-chain workers (OCW)
