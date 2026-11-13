class TransportError(Exception):
    """The remote endpoint did not accept the request."""


class RetryError(Exception):
    """A delivery attempt failed and will be tried again."""

    def __init__(self, endpoint, attempt):
        super().__init__(f"{endpoint}: attempt {attempt} failed")
        self.endpoint = endpoint
        self.attempt = attempt
