package {
    public dynamic class StaticPropertyTest {
        public var staticProp:int = 42; // A fixed property
    }
}

dynamic class SubTestClass extends StaticPropertyTest {
    public var another1:int = 100;
}

// Setup test variables
var iterations:int = 10000000; // High number of iterations for performance testing

// Create objects
var obj1:SubTestClass = new SubTestClass();

trace("start iteration");
var start:int = new Date().time;
for (var i:int = 0; i < iterations; i++) {
    obj1.staticProp;
    obj1.another1;
}
var end:int = new Date().time;
trace("Execution time: " + (end - start) + " ms");
