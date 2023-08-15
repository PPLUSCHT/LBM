; ModuleID = 'probe8.b00ed1c0-cgu.0'
source_filename = "probe8.b00ed1c0-cgu.0"
target datalayout = "e-m:e-p:32:32-p10:8:8-p20:8:8-i64:64-n32:64-S128-ni:1:10:20"
target triple = "wasm32-unknown-unknown"

; core::f64::<impl f64>::to_ne_bytes
; Function Attrs: inlinehint nounwind
define internal void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17h3eb60987a87c5e5fE"(ptr sret([8 x i8]) %0, double %self) unnamed_addr #0 {
start:
  %1 = alloca i64, align 8
  %_3 = alloca double, align 8
  store double %self, ptr %_3, align 8
  %rt = load double, ptr %_3, align 8, !noundef !0
  %2 = bitcast double %rt to i64
  store i64 %2, ptr %1, align 8
  %self1 = load i64, ptr %1, align 8, !noundef !0
  store i64 %self1, ptr %0, align 1
  ret void
}

; probe8::probe
; Function Attrs: nounwind
define hidden void @_ZN6probe85probe17h2d9e0aade4c32e22E() unnamed_addr #1 {
start:
  %_1 = alloca [8 x i8], align 1
; call core::f64::<impl f64>::to_ne_bytes
  call void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17h3eb60987a87c5e5fE"(ptr sret([8 x i8]) %_1, double 3.140000e+00) #2
  ret void
}

attributes #0 = { inlinehint nounwind "target-cpu"="generic" }
attributes #1 = { nounwind "target-cpu"="generic" }
attributes #2 = { nounwind }

!0 = !{}
