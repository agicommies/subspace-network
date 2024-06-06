// This subnet pricing mechanism is known from bittensor
// Commune uses a custom implemenentation:
// This version, makes participation more acessible, while also allowing setting decreasing subnet
// weights.

// 1. This will turn into subnet 0
// 2. 256 validators - this is a global parameter
// 3. Registration Requirements:
// Have more stake than the lowest stake in that network
// 4. Don´t include any reward distribution for this subnet
// 5. Setting Weigths:
// Validator can submit weights only once per day.
// UIDS in weight setting are the netuids of subnets.
// Subnet can not ever be deregistered, althoug it is not getting any emission.
