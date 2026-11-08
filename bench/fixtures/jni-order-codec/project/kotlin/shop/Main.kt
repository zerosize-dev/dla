package shop.codec

/** Replays generated orders through the codec; the argument is the order count. */
fun main(args: Array<String>) {
    val count = args.getOrNull(0)?.toInt() ?: 10000
    val every = maxOf(count / 4, 1)
    val stats = Stats()
    val encoder = OrderEncoder(stats)
    val generator = OrderGenerator(42)
    for (i in 0 until count) {
        val result = encoder.encode(generator.next(i))
        stats.checksumTotal += result.checksum
        if (i % every == 0) {
            println(stats.checkpoint(i))
        }
    }
    println(stats.summary())
}
