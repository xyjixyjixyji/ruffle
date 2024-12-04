package {
    public dynamic class CallTest {
        public function normalMethod():int {
            return 100;
        }
    }
}

dynamic class SubTestClass extends CallTest {
    private var _virtualValue:int = 500;

    // Virtual function with a getter
    public function get virtualFunction():Function {
        return function():int {
            return _virtualValue;
        };
    }
}

// Setup test variables
var iterations:int = 1000000; // High number of iterations for performance testing

// Create objects
var obj1:SubTestClass = new SubTestClass();

trace("start iteration");
var start:int = new Date().time;

for (var i:int = 0; i < iterations; i++) {
    // Normal method call
    obj1.normalMethod();

    // Virtual function call
    obj1.virtualFunction();
}

var end:int = new Date().time;
trace("Execution time: " + (end - start) + " ms");
