#[macro_use]
mod macros;

test!(
    undefined_function_call_is_ident_adds,
    "a {\n  color: 1 + foo();\n}\n",
    "a {\n  color: 1foo();\n}\n"
);
test!(
    unitless_plus_null,
    "a {\n  color: 1 + null;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    unitless_plus_null_plus_unitless,
    "a {\n  color: 1 + null + 1;\n}\n",
    "a {\n  color: 11;\n}\n"
);
test!(
    unit_plus_null,
    "a {\n  color: 1px + null;\n}\n",
    "a {\n  color: 1px;\n}\n"
);
test!(
    chain_ident_addition,
    "a {\n  color: a + b + c + d + e + f;\n}\n",
    "a {\n  color: abcdef;\n}\n"
);
test!(
    unquoted_plus_unquoted,
    "a {\n  color: foo + bar;\n}\n",
    "a {\n  color: foobar;\n}\n"
);
test!(
    dblquoted_plus_dblquoted,
    "a {\n  color: \"foo\" + \"bar\";\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    sglquoted_plus_sglquoted,
    "a {\n  color: 'foo' + 'bar';\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    dblquoted_plus_unquoted,
    "a {\n  color: \"foo\" + bar;\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    sglquoted_plus_unquoted,
    "a {\n  color: 'foo' + bar;\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    unquoted_plus_dblquoted,
    "a {\n  color: foo + \"bar\";\n}\n",
    "a {\n  color: foobar;\n}\n"
);
test!(
    unquoted_plus_sglquoted,
    "a {\n  color: foo + 'bar';\n}\n",
    "a {\n  color: foobar;\n}\n"
);
test!(
    sglquoted_plus_dblquoted,
    "a {\n  color: 'foo' + \"bar\";\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    dblquoted_plus_sglquoted,
    "a {\n  color: \"foo\" + 'bar';\n}\n",
    "a {\n  color: \"foobar\";\n}\n"
);
test!(
    unquoted_plus_true,
    "a {\n  color: foo + true;\n}\n",
    "a {\n  color: footrue;\n}\n"
);
test!(
    dblquoted_plus_true,
    "a {\n  color: \"foo\" + true;\n}\n",
    "a {\n  color: \"footrue\";\n}\n"
);
test!(
    sglquoted_plus_true,
    "a {\n  color: 'foo' + true;\n}\n",
    "a {\n  color: \"footrue\";\n}\n"
);
test!(
    unquoted_plus_false,
    "a {\n  color: foo + false;\n}\n",
    "a {\n  color: foofalse;\n}\n"
);
test!(
    dblquoted_plus_false,
    "a {\n  color: \"foo\" + false;\n}\n",
    "a {\n  color: \"foofalse\";\n}\n"
);
test!(
    sglquoted_plus_false,
    "a {\n  color: 'foo' + false;\n}\n",
    "a {\n  color: \"foofalse\";\n}\n"
);
test!(
    unquoted_plus_important,
    "a {\n  color: foo + !important;\n}\n",
    "a {\n  color: foo!important;\n}\n"
);
test!(
    unquoted_plus_important_uppercase,
    "a {\n  color: foo + !IMPORTANT;\n}\n",
    "a {\n  color: foo!important;\n}\n"
);
test!(
    unquoted_plus_null,
    "a {\n  color: foo + null;\n}\n",
    "a {\n  color: foo;\n}\n"
);
test!(
    dblquoted_plus_null,
    "a {\n  color: \"foo\" + null;\n}\n",
    "a {\n  color: \"foo\";\n}\n"
);
test!(
    sglquoted_plus_null,
    "a {\n  color: 'foo' + null;\n}\n",
    "a {\n  color: \"foo\";\n}\n"
);
test!(
    unquoted_plus_number_unitless,
    "a {\n  color: foo + 1;\n}\n",
    "a {\n  color: foo1;\n}\n"
);
test!(
    dblquoted_plus_number_unitless,
    "a {\n  color: \"foo\" + 1;\n}\n",
    "a {\n  color: \"foo1\";\n}\n"
);
test!(
    sglquoted_plus_number_unitless,
    "a {\n  color: 'foo' + 1;\n}\n",
    "a {\n  color: \"foo1\";\n}\n"
);
test!(
    unquoted_plus_number_unit,
    "a {\n  color: foo + 1px;\n}\n",
    "a {\n  color: foo1px;\n}\n"
);
test!(
    dblquoted_plus_number_unit,
    "a {\n  color: \"foo\" + 1px;\n}\n",
    "a {\n  color: \"foo1px\";\n}\n"
);
test!(
    sglquoted_plus_number_unit,
    "a {\n  color: 'foo' + 1px;\n}\n",
    "a {\n  color: \"foo1px\";\n}\n"
);
test!(
    true_plus_false,
    "a {\n  color: true + false;\n}\n",
    "a {\n  color: truefalse;\n}\n"
);
test!(
    true_plus_dblquoted,
    "a {\n  color: true + \"foo\";\n}\n",
    "a {\n  color: \"truefoo\";\n}\n"
);
test!(
    false_plus_dblquoted,
    "a {\n  color: false + \"foo\";\n}\n",
    "a {\n  color: \"falsefoo\";\n}\n"
);
test!(
    true_plus_unquoted,
    "a {\n  color: true + foo;\n}\n",
    "a {\n  color: truefoo;\n}\n"
);
test!(
    false_plus_unquoted,
    "a {\n  color: false + foo;\n}\n",
    "a {\n  color: falsefoo;\n}\n"
);
test!(
    true_plus_null,
    "a {\n  color: true + null;\n}\n",
    "a {\n  color: true;\n}\n"
);
test!(
    false_plus_null,
    "a {\n  color: false + null;\n}\n",
    "a {\n  color: false;\n}\n"
);
test!(
    true_plus_null_is_string,
    "a {\n  color: type-of(true + null);\n}\n",
    "a {\n  color: string;\n}\n"
);
test!(
    false_plus_null_is_string,
    "a {\n  color: type-of(false + null);\n}\n",
    "a {\n  color: string;\n}\n"
);
test!(
    null_plus_num,
    "a {\n  color: null + 1;\n}\n",
    "a {\n  color: 1;\n}\n"
);
test!(
    null_plus_num_is_string,
    "a {\n  color: type-of(null + 1);\n}\n",
    "a {\n  color: string;\n}\n"
);
test!(
    number_plus_list,
    "a {\n  color: 1 + (2 3);\n}\n",
    "a {\n  color: 12 3;\n}\n"
);
test!(
    list_plus_num,
    "a {\n  color: (1 2) + 3;\n}\n",
    "a {\n  color: 1 23;\n}\n"
);
test!(
    dblquoted_plus_list,
    "a {\n  color: \"1\" + (2 3);\n}\n",
    "a {\n  color: \"12 3\";\n}\n"
);
test!(
    list_plus_dblquoted,
    "a {\n  color: (1 2) + \"3\";\n}\n",
    "a {\n  color: \"1 23\";\n}\n"
);
test!(
    sglquoted_plus_list,
    "a {\n  color: 'a' + (b c);\n}\n",
    "a {\n  color: \"ab c\";\n}\n"
);
test!(
    list_plus_sglquoted,
    "a {\n  color: (b c) + 'a';\n}\n",
    "a {\n  color: \"b ca\";\n}\n"
);
test!(
    list_plus_list,
    "a {\n  color: (a b) + (1 2);\n}\n",
    "a {\n  color: a b1 2;\n}\n"
);
test!(
    multiple_ident_sum,
    "a {\n  color: foo + 1 + bar + 2;\n}\n",
    "a {\n  color: foo1bar2;\n}\n"
);
test!(
    dblquoted_plus_named_color,
    "a {\n  color: \"foo\" + red;\n}\n",
    "a {\n  color: \"foored\";\n}\n"
);
test!(
    sglquoted_plus_named_color,
    "a {\n  color: 'foo' + red;\n}\n",
    "a {\n  color: \"foored\";\n}\n"
);
test!(
    unquoted_plus_named_color,
    "a {\n  color: foo + red;\n}\n",
    "a {\n  color: foored;\n}\n"
);
test!(
    dblquoted_plus_hex_color,
    "a {\n  color: \"foo\" + #fff;\n}\n",
    "a {\n  color: \"foo#fff\";\n}\n"
);
test!(
    sglquoted_plus_hex_color,
    "a {\n  color: 'foo' + #fff;\n}\n",
    "a {\n  color: \"foo#fff\";\n}\n"
);
test!(
    unquoted_plus_hex_color,
    "a {\n  color: foo + #fff;\n}\n",
    "a {\n  color: foo#fff;\n}\n"
);
test!(
    number_plus_true,
    "a {\n  color: 1 + true;\n}\n",
    "a {\n  color: 1true;\n}\n"
);
test!(
    number_plus_false,
    "a {\n  color: 1 + false;\n}\n",
    "a {\n  color: 1false;\n}\n"
);
test!(
    number_plus_unary_not,
    "a {\n  color: 1 + not 2;\n}\n",
    "a {\n  color: 1false;\n}\n"
);
test!(
    number_plus_important,
    "a {\n  color: 1 + !important;\n}\n",
    "a {\n  color: 1!important;\n}\n"
);
test!(
    number_plus_arglist,
    "@function foo($a...) {
        @return 1+$a;
    }

    a {
        color: foo(a, b, c);
    }",
    "a {\n  color: 1a, b, c;\n}\n"
);
error!(
    number_plus_named_color,
    "a {\n  color: 1 + red;\n}\n", "Error: Undefined operation \"1 + red\"."
);
test!(
    double_plus,
    "a {\n  color: 1 ++ 2;\n}\n",
    "a {\n  color: 3;\n}\n"
);
test!(
    plus_div,
    "a {\n  color: 1+/2;\n}\n",
    "a {\n  color: 1/2;\n}\n"
);
test!(
    arglist_plus_number,
    "@function foo($a...) {
        @return $a + 1;
    }

    a {
        color: foo(a, b);
    }",
    "a {\n  color: a, b1;\n}\n"
);
test!(
    color_plus_ident,
    "a {\n  color: red + foo;\n}\n",
    "a {\n  color: redfoo;\n}\n"
);
test!(
    ident_plus_color,
    "a {\n  color: foo + red;\n}\n",
    "a {\n  color: foored;\n}\n"
);
test!(
    important_plus_dblquoted,
    "a {\n  color: !important + \"foo\";\n}\n",
    "a {\n  color: !importantfoo;\n}\n"
);
test!(
    important_plus_null,
    "a {\n  color: !important + null;\n}\n",
    "a {\n  color: !important;\n}\n"
);
test!(
    important_plus_unquoted,
    "a {\n  color: !important + foo;\n}\n",
    "a {\n  color: !importantfoo;\n}\n"
);
error!(
    map_lhs_add,
    "a {color: (a: b) + 1;}", "Error: (a: b) isn't a valid CSS value."
);
error!(
    map_rhs_add,
    "a {color: 1 + (a: b);}", "Error: (a: b) isn't a valid CSS value."
);
error!(
    function_lhs_add,
    "a {color: get-function(lighten) + 1;}",
    "Error: get-function(\"lighten\") isn't a valid CSS value."
);
error!(
    function_rhs_add,
    "a {color: 1 + get-function(lighten);}",
    "Error: get-function(\"lighten\") isn't a valid CSS value."
);
error!(
    add_two_calculations,
    "a {color: calc(1rem + 1px) + calc(1rem + 1px);}",
    r#"Error: Undefined operation "calc(1rem + 1px) + calc(1rem + 1px)"."#
);
error!(
    num_plus_calculation,
    "a {color: 1 + calc(1rem + 1px);}", r#"Error: Undefined operation "1 + calc(1rem + 1px)"."#
);
test!(
    quoted_string_plus_calculation,
    "a {\n  color: \"a\" + calc(1px + 1%);\n}\n",
    "a {\n  color: \"acalc(1px + 1%)\";\n}\n"
);
test!(
    calculation_plus_quoted_string,
    "a {\n  color: calc(1px + 1%) + \"a\";\n}\n",
    "a {\n  color: \"calc(1px + 1%)a\";\n}\n"
);
test!(
    empty_quoted_string_plus_calculation,
    "a {\n  color: \"\" + calc(1px + 1%);\n}\n",
    "a {\n  color: \"calc(1px + 1%)\";\n}\n"
);
test!(
    calculation_plus_empty_quoted_string,
    "a {\n  color: calc(1px + 1%) + \"\";\n}\n",
    "a {\n  color: \"calc(1px + 1%)\";\n}\n"
);
test!(
    unquoted_string_plus_calculation,
    "a {\n  color: foo + calc(1px + 1%);\n}\n",
    "a {\n  color: foocalc(1px + 1%);\n}\n"
);
test!(
    calculation_plus_unquoted_string,
    "a {\n  color: calc(1px + 1%) + foo;\n}\n",
    "a {\n  color: calc(1px + 1%)foo;\n}\n"
);
test!(
    num_plus_nan,
    "a {\n  color: 1 + (0/0);\n}\n",
    "a {\n  color: NaN;\n}\n"
);
test!(
    nan_plus_num,
    "a {\n  color: (0/0) + 1;\n}\n",
    "a {\n  color: NaN;\n}\n"
);
test!(
    nan_plus_nan,
    "a {\n  color: (0/0) + (0/0);\n}\n",
    "a {\n  color: NaN;\n}\n"
);
test!(null_plus_null, "a {\n  color: null + null;\n}\n", "");
error!(
    color_plus_number,
    "a {\n  color: red + 1;\n}\n", r#"Error: Undefined operation "red + 1"."#
);
