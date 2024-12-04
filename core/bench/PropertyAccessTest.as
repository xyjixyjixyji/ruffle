package {
    public dynamic class PropertyAccessTest {
        public var staticProp:int = 42; // A fixed property
    }
}

dynamic class SubTestClass extends PropertyAccessTest {
    public var another1:int = 100;
}

var hitrate: int = 100;

// Setup test variables
var iterations:int = 1000000; // High number of iterations for performance testing
var dynamicPropName1:String = "dynamicProp1";
var dynamicPropName2:String = "dynamicProp2";

// Create objects
var obj1:SubTestClass = new SubTestClass();
obj1.dynamicProp1 = 100;
obj1[dynamicPropName1] = 100;

trace("start iteration");
var sum:int = 0;
var start:int = new Date().time;
for (var i:int = 0; i < iterations; i++) {
    var random:int = Math.random() * 100;
    if (random < hitrate) {
        // Cache hit
        sum += obj1.staticProp;
        sum += obj1.another1;
        sum += obj1.dynamicProp1;
        sum += obj1[dynamicPropName1];
    } else {
        // Cache miss
        sum += obj1["nonExistentProp"] || 0;
        sum += obj1["nonExistentProp"] || 0;
        sum += obj1["nonExistentProp"] || 0;
        sum += obj1["nonExistentProp"] || 0;
    }
}
var end:int = new Date().time;
trace("Execution time: " + (end - start) + " ms");
