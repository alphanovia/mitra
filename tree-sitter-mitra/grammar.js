/**
 * tree-sitter grammar for Mitra, for editors (highlighting, folding,
 * structure). It follows docs/reference/language.md; the hand-written
 * parser in mitra-core stays the reference implementation.
 *
 * One `source_file` holds model commands and evidence lines alike.
 */

const sep1 = (rule, s) => seq(rule, repeat(seq(s, rule)));

module.exports = grammar({
  name: 'mitra',

  extras: $ => [/\s/, $.comment],
  word: $ => $.name,

  rules: {
    source_file: $ => repeat(choice(
      $.record, $.enum, $.opaque, $.def, $.claim, $.test, $.evidence,
    )),

    comment: _ => token(seq('--', /[^\n\r]*/)),
    name: _ => /[A-Za-z_][A-Za-z0-9_]*/,
    nat: _ => /[0-9]+/,
    number: _ => /[0-9]+(\.[0-9]+)?/,
    float: _ => /[0-9]+\.[0-9]+/,
    string: _ => /"([^"\\\n]|\\[nt"\\])*"/,

    record: $ => seq('record', field('name', $.name), $.parameters),
    enum: $ => seq('enum', field('name', $.name), optional($.type_parameters),
      '{', sep1($.variant, ','), '}'),
    type_parameters: $ => seq('(', sep1($.name, ','), ')'),
    variant: $ => seq(field('name', $.name), optional($.parameters)),
    opaque: $ => seq('opaque', field('name', $.name), $.parameters, '->', field('type', $._set)),
    def: $ => seq('def', field('name', $.name), $.parameters, '->', field('type', $._set), ':=', field('body', $._expr)),
    parameters: $ => seq('(', sep1($.parameter, ','), ')'),
    parameter: $ => seq($.name, ':', $._set),

    claim: $ => seq(
      'claim', field('name', $.name), ':',
      choice(
        $.uniform,
        $.uses,
        seq(repeat($.forall), choice($.correct, $.equation, $.property), optional($.derive)),
      ),
    ),
    uses: $ => seq(optional('only'), 'uses', field('subject', $.name), '[', sep1($.name, ','), ']'),
    uniform: $ => seq('uniform', field('subject', $.name), optional(seq('by', field('classes', $.name)))),
    correct: $ => seq('correct', '(', $.call, ')'),
    equation: $ => seq($._expr, '=', $._expr),
    property: $ => $._expr,
    derive: $ => seq(':=', 'by', 'derive', '[', sep1($.name, ','), ']'),
    call: $ => seq(field('function', $.name), optional(seq('(', sep1($._arg, ','), ')'))),
    // ASCII first; Lean's Unicode symbols are aliases.
    forall: $ => seq(choice('forall', '∀'), $.name, optional(seq(choice('in', '∈'), $._set)), ','),
    _arg: $ => choice($.name, $.boolean, $.nat, $.float, $.string, $.negative,
      $.tuple_value, $.record_value, $.list_value, $.set_value, $.map_value),
    negative: $ => seq('-', choice($.nat, $.float)),

    test: $ => seq('test', field('name', $.name), ':', repeat($.forall), $._judgment),
    evidence: $ => seq(optional($.attributes), 'evidence', repeat($.forall), $._judgment, 'at', field('confidence', $.number)),
    attributes: $ => seq('@[', sep1($.attribute, ','), ']'),
    attribute: $ => seq($.name, $.string),
    _judgment: $ => choice($.correct, $.not, $.output, $.call, $.measured),
    not: $ => seq(choice('not', '¬'), choice($.correct, $.call)),
    output: $ => seq($.call, '=', $._value),

    _set: $ => choice($.set_name, $.set_literal, $.range, $.set_op, $.set_paren,
      $.tuple_type, $.record_type, $.list_type, $.set_type, $.map_type),
    tuple_type: $ => seq('(', $._set, ',', sep1($._set, ','), ')'),
    record_type: $ => prec.right(seq(field('record', $.name), optional(seq('(', sep1($._set, ','), ')')))),
    list_type: $ => seq('List', '(', $._set, ')'),
    set_type: $ => seq('Set', '(', $._set, ')'),
    map_type: $ => seq('Map', '(', $._set, optional(seq(',', $._set)), ')'),
    set_name: _ => choice('Bool', 'Int', 'Int8', 'Int16', 'Int32', 'Int64', 'UInt8', 'UInt16',
      'UInt32', 'Float64', 'String', 'Value'),
    set_literal: $ => seq('{', optional(sep1($._value, ',')), '}'),
    range: $ => choice(
      seq($.bound, choice('..=', '..'), $.bound),
      seq($.bound, '..'),
      seq(choice('..=', '..'), $.bound),
    ),
    bound: $ => seq(optional('-'), $.nat),
    set_paren: $ => seq('(', $._set, ')'),
    set_op: $ => choice(
      prec.left(1, seq($._set, choice('|', '∪'), $._set)),
      prec.left(2, seq($._set, choice('&', '∩', '\\'), $._set)),
    ),

    _expr: $ => choice($.if, $.let, $.binary, $.unary, $.application, $.field, $.index,
      $._atom),
    field: $ => prec(8, seq($._expr, '.', field('field', choice($.name, $.nat)))),
    index: $ => prec(8, seq($._expr, '[', $._expr, ']')),
    tuple: $ => seq('(', $._expr, ',', sep1($._expr, ','), ')'),
    list: $ => seq('[', optional(sep1($._expr, ',')), ']'),
    len: $ => seq('len', '(', $._expr, ')'),
    keys: $ => seq('keys', '(', $._expr, ')'),
    set: $ => seq('{', optional(sep1($._expr, ',')), '}'),
    map: $ => choice(seq('{', ':', '}'), seq('{', sep1($.entry, ','), '}')),
    entry: $ => seq(field('key', $._expr), ':', field('value', $._expr)),
    match: $ => seq('match', $._expr, '{', sep1($.arm, ','), '}'),
    arm: $ => seq(choice('_', $.pattern), '=>', $._expr),
    pattern: $ => prec.right(seq(field('variant', $.name), optional(seq('(', sep1($.name, ','), ')')))),
    if: $ => prec.right(seq('if', $._expr, 'then', $._expr, 'else', $._expr)),
    let: $ => prec.right(seq('let', $.name, ':=', $._expr, ';', $._expr)),
    binary: $ => choice(
      ...[['||', 1], ['&&', 2], ['==', 3], ['!=', 3], ['<', 3], ['<=', 3], ['>', 3], ['>=', 3],
        ['in', 3], ['∈', 3], ['+', 4], ['-', 4], ['|', 4], ['\\', 4], ['*', 5], ['/', 5], ['&', 5]].map(([op, p]) =>
        prec.left(p, seq(field('left', $._expr), field('operator', op), field('right', $._expr)))),
    ),
    unary: $ => prec(6, seq(choice('!', '-'), $._expr)),
    application: $ => prec(7, seq(field('function', $.name), '(', sep1($._expr, ','), ')')),
    _atom: $ => choice($.name, $.boolean, $.nat, $.float, $.string, $.paren, $.tuple, $.list,
      $.len, $.keys, $.set, $.map, $.match, $._measure),
    _measure: $ => choice($.rate, $.score),
    rate: $ => seq('rate', '(', $.name, choice('in', '∈'), $._set, ',', $._expr, ')'),
    score: $ => seq(choice('precision', 'recall', 'false_positive_rate'), '(', field('subject', $.name), ')'),
    measured: $ => seq($._measure, '=', $.number),
    paren: $ => seq('(', $._expr, ')'),

    _value: $ => choice($.boolean, $.nat, $.float, $.string, $.negative_value,
      $.tuple_value, $.record_value, $.list_value, $.set_value, $.map_value, $.name),
    tuple_value: $ => seq('(', $._value, ',', sep1($._value, ','), ')'),
    record_value: $ => seq(field('record', $.name), '(', sep1($._value, ','), ')'),
    list_value: $ => seq('[', optional(sep1($._value, ',')), ']'),
    set_value: $ => seq('{', optional(sep1($._value, ',')), '}'),
    map_value: $ => choice(seq('{', ':', '}'), seq('{', sep1($.entry_value, ','), '}')),
    entry_value: $ => seq(field('key', $._value), ':', field('value', $._value)),
    negative_value: $ => seq('-', choice($.nat, $.float)),
    boolean: _ => choice('true', 'false'),
  },
});
