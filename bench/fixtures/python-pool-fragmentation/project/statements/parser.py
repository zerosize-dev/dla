def parse_line(line):
    """Split one pipe-separated statement line into a row."""
    account, posted, amount, memo, tags = line.split("|", 4)
    return {
        "account": int(account),
        "posted": posted,
        "cents": int(amount),
        "memo": memo,
        "tags": tuple(t for t in tags.split(",") if t),
    }


def parse_batch(lines):
    return [parse_line(line) for line in lines]
