import kotlinx.coroutines.*
fun a() { GlobalScope.launch { delay(1) } }
suspend fun b() { try { delay(1) } catch (e: Exception) { println(e) } finally { delay(2) } }
