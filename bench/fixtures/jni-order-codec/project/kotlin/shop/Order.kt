package shop.codec

import java.util.Random

class OrderLine(val sku: String, val quantity: Int, val priceCents: Long)

class Order(val id: String, val customer: String, val lines: List<OrderLine>) {
    fun totalCents(): Long = lines.sumOf { it.quantity * it.priceCents }
}

/** Produces a repeatable stream of orders; every eighth one is a wholesale order. */
class OrderGenerator(seed: Long) {
    private val rnd = Random(seed)
    private val skus = Array(512) { "SKU-%04d".format(it * 7) }

    fun next(index: Int): Order {
        val count = if (index % WHOLESALE_EVERY == 0) WHOLESALE_LINES else RETAIL_LINES
        val lines = List(count) {
            OrderLine(skus[rnd.nextInt(skus.size)], 1 + rnd.nextInt(9), 100L + rnd.nextInt(90000))
        }
        return Order("ord-$index", "cust-${rnd.nextInt(900)}", lines)
    }

    companion object {
        const val WHOLESALE_EVERY = 8
        const val RETAIL_LINES = 64
        const val WHOLESALE_LINES = 384
    }
}
