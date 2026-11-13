package quotes;

import java.io.ByteArrayOutputStream;
import java.io.DataOutputStream;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;

/** Writes the bytes of a class that implements {@link Rule} for one expression. */
final class ClassWriter {
    private static final int HEADER_CONSTANTS = 13;

    private ClassWriter() {}

    static byte[] write(String internalName, Node body) {
        List<Integer> numbers = new ArrayList<>();
        ByteArrayOutputStream code = new ByteArrayOutputStream();
        emit(body, code, numbers);
        code.write(0xAC); // ireturn
        byte[] method = code.toByteArray();
        try {
            ByteArrayOutputStream bytes = new ByteArrayOutputStream();
            DataOutputStream out = new DataOutputStream(bytes);
            out.writeInt(0xCAFEBABE);
            out.writeShort(0);
            out.writeShort(50);
            out.writeShort(HEADER_CONSTANTS + numbers.size() + 1);
            utf8(out, internalName);          // 1
            out.writeByte(7);                 // 2: class
            out.writeShort(1);
            utf8(out, "java/lang/Object");    // 3
            out.writeByte(7);                 // 4: class
            out.writeShort(3);
            utf8(out, "quotes/Rule");         // 5
            out.writeByte(7);                 // 6: class
            out.writeShort(5);
            utf8(out, "<init>");              // 7
            utf8(out, "()V");                 // 8
            utf8(out, "Code");                // 9
            out.writeByte(12);                // 10: name and type
            out.writeShort(7);
            out.writeShort(8);
            out.writeByte(10);                // 11: method reference
            out.writeShort(4);
            out.writeShort(10);
            utf8(out, "apply");               // 12
            utf8(out, "(II)I");               // 13
            for (int number : numbers) {
                out.writeByte(3);             // 14 and up: integer
                out.writeInt(number);
            }
            out.writeShort(0x0021);           // public, super
            out.writeShort(2);
            out.writeShort(4);
            out.writeShort(1);                // one interface
            out.writeShort(6);
            out.writeShort(0);                // no fields
            out.writeShort(2);                // two methods
            constructor(out);
            apply(out, method, Node.depth(body));
            out.writeShort(0);                // no class attributes
            return bytes.toByteArray();
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
    }

    private static void utf8(DataOutputStream out, String value) throws IOException {
        out.writeByte(1);
        out.writeUTF(value);
    }

    private static void constructor(DataOutputStream out) throws IOException {
        out.writeShort(0x0001);
        out.writeShort(7);
        out.writeShort(8);
        out.writeShort(1);
        out.writeShort(9);
        out.writeInt(12 + 5);
        out.writeShort(1);
        out.writeShort(1);
        out.writeInt(5);
        out.write(new byte[] {0x2A, (byte) 0xB7, 0, 11, (byte) 0xB1});
        out.writeShort(0);
        out.writeShort(0);
    }

    private static void apply(DataOutputStream out, byte[] code, int stack) throws IOException {
        out.writeShort(0x0001);
        out.writeShort(12);
        out.writeShort(13);
        out.writeShort(1);
        out.writeShort(9);
        out.writeInt(12 + code.length);
        out.writeShort(stack);
        out.writeShort(3);
        out.writeInt(code.length);
        out.write(code);
        out.writeShort(0);
        out.writeShort(0);
    }

    private static void emit(Node node, ByteArrayOutputStream code, List<Integer> numbers) {
        switch (node) {
            case Node.Number n -> {
                if (n.value() <= Short.MAX_VALUE) {
                    code.write(0x11); // sipush
                    code.write(n.value() >> 8);
                    code.write(n.value());
                } else {
                    int index = HEADER_CONSTANTS + numbers.size() + 1;
                    numbers.add(n.value());
                    code.write(0x13); // ldc_w
                    code.write(index >> 8);
                    code.write(index);
                }
            }
            case Node.Variable v -> code.write(0x1A + v.slot()); // iload_1, iload_2
            case Node.Binary b -> {
                emit(b.left(), code, numbers);
                emit(b.right(), code, numbers);
                code.write(switch (b.operator()) {
                    case '+' -> 0x60;
                    case '-' -> 0x64;
                    case '*' -> 0x68;
                    default -> 0x6C;
                });
            }
        }
    }
}
