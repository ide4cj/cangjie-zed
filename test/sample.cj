package sample

import std.collection.ArrayList

/** A class, a struct, an interface, an enum, an extension: every declaration the outline shows. */
public class Counter <: Countable {
    private var count: Int64 = 0

    public prop value: Int64 {
        get() {
            count
        }
    }

    public func add(step!: Int64 = 1): Unit {
        count += step
    }
}

struct Point {
    let x: Float64
    let y: Float64
}

interface Countable {}

enum Shape {
    | Circle(Float64)
    | Square(Float64)
}

extend Point {
    func norm(): Float64 {
        x * x + y * y
    }
}

type Points = ArrayList<Point>

main(): Int64 {
    let c = Counter()
    c.add(step: 2)
    let s = "count: ${c.value}\n"
    println(s)
    match (Shape.Circle(1.0)) {
        case Circle(r) => r
        case Square(a) => a
    }
    0
}
