package {
    public dynamic class DynamicPropertyTest {}
}

// Setup test variables
var iterations:int = 10000000; // High number of iterations for performance testing

// Create objects
var obj1:DynamicPropertyTest = new DynamicPropertyTest();
obj1.dynamicProp = 42;
obj1.dynamicProp2 = 100;

trace("start iteration");
var start:int = new Date().time;
for (var i:int = 0; i < iterations; i++) {
    obj1.dynamicProp;
    obj1.dynamicProp2;
}
var end:int = new Date().time;
trace("Execution time: " + (end - start) + " ms");
