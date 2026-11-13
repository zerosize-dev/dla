package quotes;

/** Defines the classes generated for pricing expressions. */
final class RuleLoader extends ClassLoader {
    RuleLoader(ClassLoader parent) {
        super("rules", parent);
    }

    /** Returns the class already defined under this name, or defines it from the bytes. */
    Class<?> definition(String name, java.util.function.Supplier<byte[]> bytes) {
        synchronized (getClassLoadingLock(name)) {
            Class<?> known = findLoadedClass(name);
            if (known != null) {
                return known;
            }
            byte[] image = bytes.get();
            return defineClass(name, image, 0, image.length);
        }
    }
}
