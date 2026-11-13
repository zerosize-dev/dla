from compression.zstd import DecompressionParameter

# Names the override documents use for the parameters of the decoder.
PARAMETER_IDS = {
    "window_log_max": DecompressionParameter.window_log_max,
}


def decoder_options(overrides):
    """Translate a tenant's override document into decoder options.

    Names this table does not know are passed through unchanged, so that a
    newer producer can add settings without a release of the reader.
    """
    return {PARAMETER_IDS.get(name, name): value for name, value in overrides.items()}
