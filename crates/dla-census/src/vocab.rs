//! Name and type vocabularies that classify declarations. Plain regex tables,
//! language-general.

use std::sync::OnceLock;

use regex::Regex;

macro_rules! re {
    ($name:ident, $pat:expr) => {
        pub fn $name() -> &'static Regex {
            static R: OnceLock<Regex> = OnceLock::new();
            R.get_or_init(|| Regex::new($pat).expect("static pattern"))
        }
    };
}

re!(
    registration_name,
    r"(?i)(listener|handler|callback|subscriber|subscription|observer|hook|watcher|registry|registration|emitter|dispatcher|interceptor)"
);
re!(
    jvm_container,
    r"\b(List|ArrayList|LinkedList|Map|HashMap|LinkedHashMap|TreeMap|SortedMap|NavigableMap|ConcurrentHashMap|ConcurrentMap|ConcurrentSkipListMap|WeakHashMap|IdentityHashMap|EnumMap|Set|HashSet|LinkedHashSet|TreeSet|SortedSet|ConcurrentSkipListSet|Queue|Deque|ArrayDeque|PriorityQueue|BlockingQueue|BlockingDeque|LinkedBlockingQueue|LinkedBlockingDeque|ArrayBlockingQueue|ConcurrentLinkedQueue|ConcurrentLinkedDeque|CopyOnWriteArrayList|CopyOnWriteArraySet|Collection|Vector|Stack|Hashtable|Cache|LoadingCache|AsyncCache|Multimap|ListMultimap|SetMultimap|Table|BitSet|MutableList|MutableMap|MutableSet|MutableCollection|ArrayList|Array|IntArray|LongArray|ByteArray|Iterable|Properties|StringBuffer|StringBuilder)\b|\[\]|\w*(Cache|Registry|Pool|Queue|Store|Chain|Buffer|Holder|Table|Index|Memo|Bus)$"
);
re!(
    jvm_scalar,
    r"^(?:final\s+)?(byte|short|int|long|float|double|boolean|char|Byte|Short|Integer|Long|Float|Double|Boolean|Character|String|CharSequence|Logger|Log|Class|Pattern|Duration|Instant|LocalDate|LocalDateTime|UUID|BigDecimal|BigInteger|Charset|Random|Object|Void|Number|AtomicInteger|AtomicLong|AtomicBoolean|Lock|ReentrantLock|Level|TimeUnit|Locale|Clock|SecureRandom|Gson|ObjectMapper)(<.*>)?$"
);
re!(
    jvm_functional,
    r"\b(Runnable|Callable|Supplier|Consumer|BiConsumer|Function|BiFunction|Predicate|BiPredicate|Comparator|Executor)\b"
);
re!(
    jvm_thread_local,
    r"\b(ThreadLocal|InheritableThreadLocal|FastThreadLocal)\b"
);
re!(
    jvm_threadish,
    r"\b(Thread|Timer|ExecutorService|ScheduledExecutorService|ThreadPoolExecutor|ScheduledThreadPoolExecutor|ForkJoinPool|CompletableFuture|Future|ScheduledFuture|CoroutineScope|Job)\b"
);
re!(
    jvm_lockish_name,
    r"(?i)^(lock|mutex|monitor|log|logger|serialversionuid|random|rng|gson|mapper)$"
);
re!(
    longlived_type_name,
    r"(?:Registry|Cache|Pool|Manager|Service|Server|Store|Bus|Hub|Broker|Scheduler|Engine|Dispatcher|Tracker|Interner|Arena|Supervisor|Controller|Repository|Gateway|Router|Ledger|Runtime|Reactor|Loop|Daemon|Singleton|Global|Shared|Executor|Selector|Allocator|Registrar)s?$"
);
