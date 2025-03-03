@_cdecl("__swift_bridge__$get_test")
func __swift_bridge__get_test (_ provider: UnsafeMutableRawPointer) -> __swift_bridge__$TestFFIResult {
    getTest(provider: Unmanaged<TestFFIProviderRef>.fromOpaque(provider).takeRetainedValue()).intoFfiRepr()
}

public struct TestFFI {
    public var value: Int32
    public var other_value: Double

    public init(value: Int32,other_value: Double) {
        self.value = value
        self.other_value = other_value
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$TestFFI {
        { let val = self; return __swift_bridge__$TestFFI(value: val.value, other_value: val.other_value); }()
    }
}
extension __swift_bridge__$TestFFI {
    @inline(__always)
    func intoSwiftRepr() -> TestFFI {
        { let val = self; return TestFFI(value: val.value, other_value: val.other_value); }()
    }
}
extension __swift_bridge__$Option$TestFFI {
    @inline(__always)
    func intoSwiftRepr() -> Optional<TestFFI> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<TestFFI>) -> __swift_bridge__$Option$TestFFI {
        if let v = val {
            return __swift_bridge__$Option$TestFFI(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$TestFFI(is_some: false, val: __swift_bridge__$TestFFI())
        }
    }
}
public struct TestFFIResult {
    public var success: Bool
    public var data: TestFFI

    public init(success: Bool,data: TestFFI) {
        self.success = success
        self.data = data
    }

    @inline(__always)
    func intoFfiRepr() -> __swift_bridge__$TestFFIResult {
        { let val = self; return __swift_bridge__$TestFFIResult(success: val.success, data: val.data.intoFfiRepr()); }()
    }
}
extension __swift_bridge__$TestFFIResult {
    @inline(__always)
    func intoSwiftRepr() -> TestFFIResult {
        { let val = self; return TestFFIResult(success: val.success, data: val.data.intoSwiftRepr()); }()
    }
}
extension __swift_bridge__$Option$TestFFIResult {
    @inline(__always)
    func intoSwiftRepr() -> Optional<TestFFIResult> {
        if self.is_some {
            return self.val.intoSwiftRepr()
        } else {
            return nil
        }
    }

    @inline(__always)
    static func fromSwiftRepr(_ val: Optional<TestFFIResult>) -> __swift_bridge__$Option$TestFFIResult {
        if let v = val {
            return __swift_bridge__$Option$TestFFIResult(is_some: true, val: v.intoFfiRepr())
        } else {
            return __swift_bridge__$Option$TestFFIResult(is_some: false, val: __swift_bridge__$TestFFIResult())
        }
    }
}

@_cdecl("__swift_bridge__$TestFFIProvider$_free")
func __swift_bridge__TestFFIProvider__free (ptr: UnsafeMutableRawPointer) {
    let _ = Unmanaged<TestFFIProvider>.fromOpaque(ptr).takeRetainedValue()
}



