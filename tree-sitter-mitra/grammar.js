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
      $.opaque, $.def, $.claim, $.test, $.profile, $.policy, $.evidence,
    )),

    comment: _ => token(seq('--', /[^\n\r]*/)),
    name: _ => /[A-Za-z_][A-Za-z0-9_]*/,
    nat: _ => /[0-9]+/,
    number: _ => /[0-9]+(\.[0-9]+)?/,
    float: _ => /[0-9]+\.[0-9]+/,
    string: _ => /"[^"\n]*"/,

    opaque: $ => seq('opaque', field('name', $.name), repeat1($.binder), ':', field('type', $._set)),
    def: $ => seq('def', field('name', $.name), repeat1($.binder), ':', field('type', $._set), ':=', field('body', $._expr)),
    binder: $ => seq('(', repeat1($.name), ':', $._set, ')'),

    claim: $ => seq(
      'claim', field('name', $.name), ':',
      choice(
        $.uniform,
        seq(repeat($.forall), choice($.correct, $.equation), optional($.derive)),
      ),
    ),
    uniform: $ => seq('uniform', field('subject', $.name), 'by', field('classes', $.name)),
    correct: $ => seq('correct', '(', $.call, ')'),
    equation: $ => seq($.call, '=', $._expr),
    derive: $ => seq(':=', 'by', 'derive', '[', sep1($.name, ','), ']'),
    call: $ => seq(field('function', $.name), repeat($._arg)),
    // ASCII first; Lean's Unicode symbols are aliases.
    forall: $ => seq(choice('forall', '∀'), $.name, optional(seq(choice('in', '∈'), $._set)), ','),
    _arg: $ => choice($.name, $.boolean, $.nat, $.float, $.string, $.negative),
    negative: $ => seq('(', '-', choice($.nat, $.float), ')'),

    profile: $ => seq('profile', field('name', $.name), 'for', field('subject', $.name), repeat($.region)),
    region: $ => seq('|', choice('_', seq($.name, choice('in', '∈'), $._set)), '=>', $.number),
    policy: $ => seq('policy', choice(
      'strict',
      seq(field('name', $.name), ':=', choice('strict', seq('coverage', choice($.name, 'uniform')))),
    )),

    test: $ => seq('test', field('name', $.name), ':', repeat($.forall), $._judgment),
    evidence: $ => seq(optional($.attributes), 'evidence', repeat($.forall), $._judgment, 'at', field('confidence', $.number)),
    attributes: $ => seq('@[', sep1($.attribute, ','), ']'),
    attribute: $ => seq($.name, $.string),
    _judgment: $ => choice($.correct, $.not, $.output, $.call),
    not: $ => seq(choice('not', '¬'), choice($.correct, $.call)),
    output: $ => seq($.call, '=', $._value),

    _set: $ => choice($.set_name, $.set_literal, $.range, $.set_op, $.set_paren),
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

    _expr: $ => choice($.if, $.let, $.binary, $.unary, $.application, $._atom),
    if: $ => prec.right(seq('if', $._expr, 'then', $._expr, 'else', $._expr)),
    let: $ => prec.right(seq('let', $.name, ':=', $._expr, ';', $._expr)),
    binary: $ => choice(
      ...[['||', 1], ['&&', 2], ['==', 3], ['!=', 3], ['<', 3], ['<=', 3], ['>', 3], ['>=', 3],
        ['+', 4], ['-', 4], ['*', 5]].map(([op, p]) =>
        prec.left(p, seq(field('left', $._expr), field('operator', op), field('right', $._expr)))),
    ),
    unary: $ => prec(6, seq(choice('!', '-'), $._expr)),
    application: $ => prec(7, seq(field('function', $.name), repeat1($._atom))),
    _atom: $ => choice($.name, $.boolean, $.nat, $.float, $.string, $.paren),
    paren: $ => seq('(', $._expr, ')'),

    _value: $ => choice($.boolean, $.nat, $.float, $.string, $.negative_value),
    negative_value: $ => seq('-', choice($.nat, $.float)),
    boolean: _ => choice('true', 'false'),
  },
});
