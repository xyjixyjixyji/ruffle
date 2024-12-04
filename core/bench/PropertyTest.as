package {
    public dynamic class PropertyTest {
        public var staticProp:int = 42; // A fixed property
    }
}

dynamic class SubTestClass extends PropertyTest {
    public var another1:int = 100;
}

// Setup test variables
var iterations:int = 10000000; // High number of iterations for performance testing

// Create objects
var obj1:SubTestClass = new SubTestClass();
obj1.dynamicProp = 42;
obj1.dynamicProp2 = 100;

trace("start iteration");
var start:int = new Date().time;
for (var i:int = 0; i < iterations; i++) {
    obj1.staticProp;
    obj1.another1;
    obj1.dynamicProp;
    obj1.dynamicProp2;
}
var end:int = new Date().time;
trace("Execution time: " + (end - start) + " ms");
