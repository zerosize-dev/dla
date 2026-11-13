package tally

// DaysKept is the number of daily buckets every account has.
const DaysKept = 365

// Ledger keeps the net amount per account and day.
type Ledger struct {
	accounts map[string][]int64
}

// NewLedger returns an empty ledger.
func NewLedger() *Ledger {
	return &Ledger{accounts: make(map[string][]int64)}
}

// Open makes sure an account exists, with all its daily buckets.
func (l *Ledger) Open(account string) {
	if _, ok := l.accounts[account]; !ok {
		l.accounts[account] = make([]int64, DaysKept)
	}
}

// Apply adds an event to its account's daily bucket. It reports false for an
// account that was never opened or a day outside the range.
func (l *Ledger) Apply(ev Event) bool {
	days, ok := l.accounts[ev.Account]
	if !ok || ev.Day < 0 || ev.Day >= DaysKept {
		return false
	}
	if ev.Kind == Debit {
		days[ev.Day] -= ev.Cents
	} else {
		days[ev.Day] += ev.Cents
	}
	return true
}

// Accounts reports how many accounts are open.
func (l *Ledger) Accounts() int {
	return len(l.accounts)
}

// Balance returns the sum of an account's daily buckets.
func (l *Ledger) Balance(account string) int64 {
	var sum int64
	for _, v := range l.accounts[account] {
		sum += v
	}
	return sum
}
