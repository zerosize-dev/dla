"""Small helpers for turning ledger values into printable text."""

CURRENCY_SYMBOLS = {"USD": "$", "EUR": "EUR ", "GBP": "GBP "}


def money(cents, currency="USD"):
    sign = "-" if cents < 0 else ""
    whole, frac = divmod(abs(cents), 100)
    symbol = CURRENCY_SYMBOLS.get(currency, currency + " ")
    return f"{sign}{symbol}{whole:,}.{frac:02d}"


def pad(text, width):
    if len(text) >= width:
        return text[:width]
    return text + " " * (width - len(text))


def title(account, month):
    return f"Account {account:06d} - {month}"


def checksum(text):
    value = 17
    for ch in text:
        value = (value * 31 + ord(ch)) & 0xFFFFFFFF
    return value
