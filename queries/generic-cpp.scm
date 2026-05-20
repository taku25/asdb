; generic-cpp.scm — built-in C++ symbol extraction queries

(class_specifier
  name: (type_identifier) @symbol.name
  body: (_)
  (#set! symbol.type "class"))

(struct_specifier
  name: (type_identifier) @symbol.name
  body: (_)
  (#set! symbol.type "struct"))

(enum_specifier
  name: (type_identifier) @symbol.name
  (#set! symbol.type "enum"))

(function_definition
  declarator: (function_declarator
    declarator: (identifier) @symbol.name)
  (#set! symbol.type "function"))

(declaration
  type: (_)
  declarator: (function_declarator
    declarator: (identifier) @symbol.name)
  (#set! symbol.type "function"))
