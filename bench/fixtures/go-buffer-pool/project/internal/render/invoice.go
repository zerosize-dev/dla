package render

import (
	"strconv"
)

// Line is one billed line of an invoice.
type Line struct {
	SKU       string
	Desc      string
	Quantity  int
	UnitCents int64
}

// Invoice is the input of the renderer.
type Invoice struct {
	ID       string
	Customer string
	Currency string
	Lines    []Line
}

// Total returns the invoice total in cents.
func (inv Invoice) Total() int64 {
	var t int64
	for _, l := range inv.Lines {
		t += int64(l.Quantity) * l.UnitCents
	}
	return t
}

// appendMoney appends a cents amount as a decimal with two fractional digits.
func appendMoney(dst []byte, cents int64) []byte {
	if cents < 0 {
		dst = append(dst, '-')
		cents = -cents
	}
	dst = strconv.AppendInt(dst, cents/100, 10)
	dst = append(dst, '.')
	frac := cents % 100
	if frac < 10 {
		dst = append(dst, '0')
	}
	return strconv.AppendInt(dst, frac, 10)
}

// appendInvoice writes the text form of an invoice to dst.
func appendInvoice(dst []byte, inv Invoice) []byte {
	dst = append(dst, "INVOICE "...)
	dst = append(dst, inv.ID...)
	dst = append(dst, "\nCustomer: "...)
	dst = append(dst, inv.Customer...)
	dst = append(dst, '\n')
	for i, l := range inv.Lines {
		dst = strconv.AppendInt(dst, int64(i+1), 10)
		dst = append(dst, ". "...)
		dst = append(dst, l.SKU...)
		dst = append(dst, ' ')
		dst = append(dst, l.Desc...)
		dst = append(dst, " x"...)
		dst = strconv.AppendInt(dst, int64(l.Quantity), 10)
		dst = append(dst, " @ "...)
		dst = appendMoney(dst, l.UnitCents)
		dst = append(dst, '\n')
	}
	dst = append(dst, "TOTAL "...)
	dst = appendMoney(dst, inv.Total())
	dst = append(dst, ' ')
	dst = append(dst, inv.Currency...)
	dst = append(dst, '\n')
	return dst
}
