/*
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use common::SourceLocationKey;
use fixture_tests::Fixture;
use graphql_ir::{build, Program};
use graphql_syntax::parse_executable;
use graphql_test_helpers::diagnostics_to_sorted_string;
use graphql_transforms::{required_directive, FeatureFlags};
use interner::Intern;
use relay_codegen::{print_fragment, print_operation};
use std::sync::Arc;
use test_schema::{get_test_schema, get_test_schema_with_extensions};

pub fn transform_fixture(fixture: &Fixture<'_>) -> Result<String, String> {
    let parts: Vec<_> = fixture.content.split("%extensions%").collect();
    let (base, schema) = match parts.as_slice() {
        [base, extensions] => (base, get_test_schema_with_extensions(extensions)),
        [base] => (base, get_test_schema()),
        _ => panic!("Invalid fixture input {}", fixture.content),
    };

    let ast = parse_executable(base, SourceLocationKey::standalone(fixture.file_name)).unwrap();
    let ir = build(&schema, &ast.definitions)
        .map_err(|diagnostics| diagnostics_to_sorted_string(fixture.content, &diagnostics))?;
    let program = Program::from_definitions(Arc::clone(&schema), ir);

    required_directive(
        &program,
        &FeatureFlags {
            enable_required_transform_for_prefix: Some("".intern()),
            enable_flight_transform: false,
        },
    )
    .map(|next_program| {
        next_program
            .fragments()
            .map(|def| print_fragment(&schema, &def))
            .chain(
                next_program
                    .operations()
                    .map(|def| print_operation(&schema, &def)),
            )
            .collect::<Vec<_>>()
            .join("\n\n")
    })
    .map_err(|diagnostics| diagnostics_to_sorted_string(fixture.content, &diagnostics))
}
