#include "toy/Dialects.h"

#include "toy/Dialect.h"
#include "mlir/CAPI/Registration.h"

MLIR_DEFINE_CAPI_DIALECT_REGISTRATION(Toy, toy,
                                      mlir::toy::ToyDialect)
