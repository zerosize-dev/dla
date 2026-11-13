from .models import AccountTotal


def totals_by_account(rows):
    sums = {}
    for row in rows:
        count, cents = sums.get(row["account"], (0, 0))
        sums[row["account"]] = (count + 1, cents + row["cents"])
    return [AccountTotal(a, c, t) for a, (c, t) in sorted(sums.items())]


def flagged(rows, limit_cents):
    return [row for row in rows if abs(row["cents"]) >= limit_cents]
