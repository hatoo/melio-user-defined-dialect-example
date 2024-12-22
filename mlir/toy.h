
#ifndef MLIR_C_DIALECT_TOY_H
#define MLIR_C_DIALECT_TOY_H

#include "mlir-c/IR.h"
#include "mlir/CAPI/Registration.h"

#ifdef __cplusplus
extern "C"
{
#endif

    MLIR_DECLARE_CAPI_DIALECT_REGISTRATION(TOY, toy);

#ifdef __cplusplus
}
#endif

MLIR_DEFINE_CAPI_DIALECT_REGISTRATION(Toy, toy, mlir::toy::ToyDialect)

#endif // MLIR_C_DIALECT_SCF_H