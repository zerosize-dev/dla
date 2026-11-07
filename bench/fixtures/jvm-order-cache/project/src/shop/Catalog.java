package shop;

import java.util.List;

/** The fixed product list the storefront sells. */
final class Catalog {
    record Product(String sku, String title, long priceCents) {}

    private static final List<Product> PRODUCTS = List.of(
            new Product("SKU-100", "Notebook", 450),
            new Product("SKU-101", "Fountain pen", 1895),
            new Product("SKU-102", "Desk lamp", 3290),
            new Product("SKU-103", "Paper clips (200)", 275),
            new Product("SKU-104", "Stapler", 1240),
            new Product("SKU-105", "Whiteboard", 8990),
            new Product("SKU-106", "Marker set", 960),
            new Product("SKU-107", "Binder", 515),
            new Product("SKU-108", "Sticky notes", 380),
            new Product("SKU-109", "Monitor stand", 4750),
            new Product("SKU-110", "Cable tray", 2210),
            new Product("SKU-111", "Label printer", 11900));

    private Catalog() {}

    static Product at(int index) {
        return PRODUCTS.get(Math.floorMod(index, PRODUCTS.size()));
    }

    static int size() {
        return PRODUCTS.size();
    }
}
