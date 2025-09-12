#[macro_use]
mod macros;

test!(
    for_1_through_3,
    "@for $i from 1 through 3 {\n  a {\n    color: $i;\n  }\n}\n",
    "a {\n  color: 1;\n}\n\na {\n  color: 2;\n}\n\na {\n  color: 3;\n}\n"
);
test!(
    for_1_to_3,
    "@for $i from 1 to 3 {\n  a {\n    color: $i;\n  }\n}\n",
    "a {\n  color: 1;\n}\n\na {\n  color: 2;\n}\n"
);
test!(
    for_3_through_1,
    "@for $i from 3 through 1 {\n  a {\n    color: $i;\n  }\n}\n",
    "a {\n  color: 3;\n}\n\na {\n  color: 2;\n}\n\na {\n  color: 1;\n}\n"
);
test!(
    for_3_to_1,
    "@for $i from 3 to 1 {\n  a {\n    color: $i;\n  }\n}\n",
    "a {\n  color: 3;\n}\n\na {\n  color: 2;\n}\n"
);
test!(
    for_var_through_var,
    "$a: 1;\n$b: 3;\n@for $x from $a through $b {\n  div {\n    color: $x;\n  }\n}\n",
    "div {\n  color: 1;\n}\n\ndiv {\n  color: 2;\n}\n\ndiv {\n  color: 3;\n}\n"
);
test!(
    for_var_decl,
    "@for $x from 1 to 3 {\n  $limit: $x;\n\n  a {\n    color: $limit;\n  }\n}\n",
    "a {\n  color: 1;\n}\n\na {\n  color: 2;\n}\n"
);
test!(
    for_styles,
    "a {\n  @for $i from 1 to 3 {\n    color: $i;\n  }\n}\n",
    "a {\n  color: 1;\n  color: 2;\n}\n"
);
test!(
    scope,
    "a {\n  $a: red;\n  @for $i from 1 to 3 {\n    $a: blue;\n  }\n  color: $a;\n}\n",
    "a {\n  color: blue;\n}\n"
);
test!(
    simple_return_in_function,
    "@function foo() {\n  @for $i from 1 to 2 {\n    @return $i;\n  }\n}\na {\n  color: foo();\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    return_gated_by_if_in_function,
    "@function foo() {\n  @for $i from 1 through 2 {\n    @if $i==2 {\n      @return $i;\n    }\n  }\n}\na {\n  color: foo();\n}\n",
    "a {\n  color: 2;\n}\n"
);
test!(
    inner_if,
    "a {\n  @for $i from 1 to 3 {\n    @if $i==2 {\n      color: red;\n    }\n  }\n}\n",
    "a {\n  color: red;\n}\n"
);
test!(
    from_negative_to_positive,
    "@for $i from -1 to 3 {\n    a {\n        color: red;\n    }\n}\n",
    "a {\n  color: red;\n}\n\na {\n  color: red;\n}\n\na {\n  color: red;\n}\n\na {\n  color: red;\n}\n"
);
test!(
    from_negative_to_negative,
    "@for $i from -1 to -3 {\n    a {\n        color: red;\n    }\n}\n",
    "a {\n  color: red;\n}\n\na {\n  color: red;\n}\n"
);
test!(
    variable_named_to_as_value,
    "$to: 0;

    @for $i from $to to 1 {
        a {
            color: red;
        }
    }",
    "a {\n  color: red;\n}\n"
);
test!(
    to_crazy_interpolation,
    "a {
        @for $i from 0 to length(#{\"#{\"\\\\}}}{{{\"}#\"}) {
            color: #{\"#{\"\\\\}}}{{{\"}#\"};
        }
    }",
    "a {\n  color: \\}}}{{{#;\n}\n"
);
test!(
    from_crazy_interpolation,
    "a {
        @for $i from length(#{\"#{\"\\\\}}}{{{\"}#\"}) to 2 {
            color: #{\"#{\"\\\\}}}{{{\"}#\"};
        }
    }",
    "a {\n  color: \\}}}{{{#;\n}\n"
);
test!(
    indexing_variable_does_not_affect_outer_scopes,
    "a {
        $a: 1;
        $b: 1;

        @for $a from 1 through 2 {
            color: $a;
            $b: $a;
        }

        color: $a;
        color: $b;
    }",
    "a {\n  color: 1;\n  color: 2;\n  color: 1;\n  color: 2;\n}\n"
);
test!(
    multiline_comments_everywhere,
    "  /**/  @for  /**/  $i  /**/  from  /**/  0  /**/  to  /**/  2  /**/  {}  /**/  ",
    "/**/\n/**/\n"
);
test!(
    uppercase_keywords,
    "@for $i FROM 0 TO 2 {
        @foo;
    }",
    "@foo;\n@foo;\n"
);
test!(
    escaped_keyword_through,
    r"@for $i from 0 \74 hrough 0 {
      a {
        color: \74;
      }
    }",
    "a {\n  color: t;\n}\n"
);
test!(
    escaped_keyword_to,
    r"@for $i from 0 \74 o 1 {
      a {
        color: \74;
      }
    }",
    "a {\n  color: t;\n}\n"
);
test!(
    escaped_keyword_from_lower,
    r"@for $i \66rom 0 to 1 {
      a {
        color: \74;
      }
    }",
    "a {\n  color: t;\n}\n"
);
test!(
    escaped_keyword_from_upper,
    r"@for $i \46rom 0 to 1 {
      a {
        color: \74;
      }
    }",
    "a {\n  color: t;\n}\n"
);
test!(
    variable_name_is_keyword,
    r"$to: 0;
    @for $from from $to to 1 {
      a {
        color: red;
      }
    }",
    "a {\n  color: red;\n}\n"
);
error!(
    missing_keyword_from,
    "@for $i fro", "Error: Expected \"from\"."
);
error!(missing_dollar_sign, "@for", "Error: expected \"$\".");
error!(
    variable_missing_identifier,
    "@for $", "Error: Expected identifier."
);
error!(
    from_value_is_decimal,
    "@for $i from 0.5 to 2 {}", "Error: 0.5 is not an int."
);
error!(
    to_value_is_decimal,
    "@for $i from 0 to 1.5 {}", "Error: 1.5 is not an int."
);
error!(
    from_value_is_non_numeric,
    "@for $i from red to 2 {}", "Error: red is not a number."
);
error!(
    to_value_is_non_numeric,
    "@for $i from 0 to red {}", "Error: red is not a number."
);
error!(
    from_nan,
    "@for $i from (0/0) through 0 {}", "Error: NaN is not an int."
);
error!(
    to_nan,
    "@for $i from 0 through (0/0) {}", "Error: NaN is not an int."
);
test!(
    to_and_from_i32_min,
    "@for $i from -2147483648 through -2147483648 {}",
    ""
);
test!(
    to_and_from_comparable_units,
    "@for $i from 1px to (3px * 1in) / 1in {
        a {
            color: $i;
        }
    }",
    "a {\n  color: 1px;\n}\n\na {\n  color: 2px;\n}\n"
);
error!(
    to_and_from_non_comparable_units,
    "@for $i from 1px to 2vh {
        a {
            color: $i;
        }
    }",
    "Error: to and from values have incompatible units"
);
error!(
    invalid_escape_sequence_in_declaration,
    "@for $i from 0 \\110000 o 2 {}", "Error: Invalid Unicode code point."
);
error!(
    invalid_keyword_after_first_number,
    "@for $i from 1 FOO 3 {}", "Error: Expected \"to\" or \"through\"."
);
test!(
    to_equals_from,
    "@for $i from 5 to 5 {
        a {
            color: red;
        }
    }",
    ""
);
