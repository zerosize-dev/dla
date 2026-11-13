module example.com/dispatchd

go 1.24

require example.invalid/ledgerkit v1.4.2

// The ledger client is published to the company's module proxy; CI builds
// resolve it from the vendor checkout next to this repository.
replace example.invalid/ledgerkit => ../vendor/ledgerkit
