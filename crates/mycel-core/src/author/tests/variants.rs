use super::*;
use crate::author::types::{
    MergeReasonBranchKind, MergeReasonKind, MergeReasonSubjectKind, MergeReasonVariantKind,
};

#[test]
fn merge_authoring_reports_multi_variant_when_parents_disagree() {
    let store_root = temp_dir("merge-multi-variant");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            title: "Merge Variant".to_string(),
            language: "en".to_string(),
            timestamp: 10,
        },
    )
    .expect("document should be created");

    let base_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: document.genesis_revision_id.clone(),
            timestamp: 11,
            ops: json!([
                {
                    "op": "insert_block",
                    "new_block": {
                        "block_id": "blk:merge-001",
                        "block_type": "paragraph",
                        "content": "Base",
                        "attrs": {},
                        "children": []
                    }
                }
            ]),
        },
    )
    .expect("base patch should be created");
    let base_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![document.genesis_revision_id.clone()],
            patches: vec![base_patch.patch_id],
            merge_strategy: None,
            timestamp: 12,
        },
    )
    .expect("base revision should be committed");

    let left_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 13,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-001",
                    "new_content": "Left variant"
                }
            ]),
        },
    )
    .expect("left patch should be created");
    let left_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![left_patch.patch_id],
            merge_strategy: None,
            timestamp: 14,
        },
    )
    .expect("left revision should be committed");

    let right_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 15,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-001",
                    "new_content": "Right variant"
                }
            ]),
        },
    )
    .expect("right patch should be created");
    let right_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![right_patch.patch_id],
            merge_strategy: None,
            timestamp: 16,
        },
    )
    .expect("right revision should be committed");

    let center_patch = create_patch_in_store(
        &store_root,
        &signing_key,
        &PatchCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            base_revision: base_revision.revision_id.clone(),
            timestamp: 17,
            ops: json!([
                {
                    "op": "replace_block",
                    "block_id": "blk:merge-001",
                    "new_content": "Center variant"
                }
            ]),
        },
    )
    .expect("center patch should be created");
    let center_revision = commit_revision_to_store(
        &store_root,
        &signing_key,
        &RevisionCommitParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![base_revision.revision_id.clone()],
            patches: vec![center_patch.patch_id],
            merge_strategy: None,
            timestamp: 18,
        },
    )
    .expect("center revision should be committed");

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-variant".to_string(),
            parents: vec![
                left_revision.revision_id,
                right_revision.revision_id,
                center_revision.revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-variant".to_string(),
                blocks: vec![paragraph_block("blk:merge-001", "Right variant")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 19,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains("adopted a non-primary parent replacement")),
        "expected multi-variant reason, got {summary:?}"
    );
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains(
                "selected one non-primary replacement while other competing non-primary replacements remained"
            )),
        "expected competing-variant reason, got {summary:?}"
    );
    let content_selection_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:merge-001"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::Content
        })
        .expect("expected structured content selection detail");
    assert_eq!(
        content_selection_detail.subject_kind,
        MergeReasonSubjectKind::Block
    );
    assert!(
        content_selection_detail
            .primary_variant
            .contains("Left variant"),
        "expected primary variant detail, got {content_selection_detail:?}"
    );
    assert!(
        content_selection_detail
            .resolved_variant
            .contains("Right variant"),
        "expected resolved variant detail, got {content_selection_detail:?}"
    );
    assert_eq!(content_selection_detail.competing_variants.len(), 1);
    assert!(
        content_selection_detail
            .competing_variants
            .iter()
            .any(|variant| variant.contains("Center variant")),
        "expected competing center variant detail, got {content_selection_detail:?}"
    );
    assert_eq!(
        content_selection_detail.branch_kind,
        Some(MergeReasonBranchKind::AdoptedNonPrimaryReplacement)
    );
    let content_competing_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:merge-001"
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.variant_kind == MergeReasonVariantKind::Content
        })
        .expect("expected structured competing content detail");
    assert_eq!(
        content_competing_detail.branch_kind,
        Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacements)
    );
    assert_eq!(
        content_competing_detail.competing_variants.len(),
        2,
        "expected both non-primary content alternatives, got {content_competing_detail:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_metadata_parents_disagree() {
    let store_root = temp_dir("merge-metadata-variant");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-variant".to_string(),
            title: "Merge Metadata Variant".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let left_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-variant",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "left"
                }
            }
        ]),
    );
    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-variant",
        &document.genesis_revision_id,
        23,
        24,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-variant",
        &document.genesis_revision_id,
        25,
        26,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "center"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-variant".to_string(),
            parents: vec![left_revision_id, right_revision_id, center_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-variant".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 27,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason
            .contains("metadata key 'topic' adopted a non-primary parent replacement")),
        "expected metadata multi-variant reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason
            .contains("metadata key 'topic' selected one non-primary replacement while other competing non-primary replacements remained")),
        "expected competing metadata reason, got {summary:?}"
    );
    let metadata_selection_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "topic"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected structured metadata selection detail");
    assert_eq!(
        metadata_selection_detail.subject_kind,
        MergeReasonSubjectKind::MetadataKey
    );
    assert_eq!(metadata_selection_detail.primary_variant, "\"left\"");
    assert_eq!(metadata_selection_detail.resolved_variant, "\"right\"");
    assert_eq!(
        metadata_selection_detail.competing_variants,
        vec!["\"center\"".to_string()]
    );
    assert_eq!(
        metadata_selection_detail.branch_kind,
        Some(MergeReasonBranchKind::AdoptedNonPrimaryReplacement)
    );
    let metadata_competing_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "topic"
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected structured competing metadata detail");
    assert_eq!(
        metadata_competing_detail.branch_kind,
        Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacements)
    );
    assert_eq!(
        metadata_competing_detail.competing_variants,
        vec!["\"center\"".to_string(), "\"right\"".to_string(),],
        "expected all non-primary metadata alternatives, got {metadata_competing_detail:?}"
    );
    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let ops = patch_value["ops"]
        .as_array()
        .expect("generated patch ops should be an array");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["op"], "set_metadata");
    assert_eq!(ops[0]["metadata"]["topic"], "right");
    assert!(
        ops[0].get("entries").is_none(),
        "merge-generated set_metadata op should use parser-compatible metadata field"
    );
    let patch = parse_patch_object(&patch_value).expect("generated patch should parse");
    assert!(patch.ops.iter().any(|op| matches!(
        op,
        PatchOperation::SetMetadata { entries }
        if entries.get("topic") == Some(&Value::String("right".to_string()))
    )));

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_content_replacements() {
    let store_root = temp_dir("merge-content-duplicate-non-primary-replacements");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-duplicate-non-primary-replacements".to_string(),
            title: "Merge Content Duplicate Non Primary Replacements".to_string(),
            language: "en".to_string(),
            timestamp: 28,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-duplicate-non-primary-replacements",
        &document.genesis_revision_id,
        29,
        30,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-duplicate-choice",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-duplicate-non-primary-replacements",
        &base_revision_id,
        31,
        32,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-duplicate-choice",
                "new_content": "Right variant"
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-duplicate-non-primary-replacements",
        &base_revision_id,
        33,
        34,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-duplicate-choice",
                "new_content": "Right variant"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-duplicate-non-primary-replacements".to_string(),
            parents: vec![base_revision_id, right_revision_id, center_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-duplicate-non-primary-replacements".to_string(),
                blocks: vec![paragraph_block(
                    "blk:merge-content-duplicate-choice",
                    "Right variant",
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 35,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-duplicate-choice' selected one non-primary replacement while other competing non-primary replacements remained"
        )),
        "expected competing content replacement reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-duplicate-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.competing_variants.len() == 1
                && detail.competing_variants[0].contains("Right variant")
        }),
        "expected selected content replacement detail to retain one remaining duplicate, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-duplicate-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacements)
                && detail.competing_variants.len() == 2
                && detail
                    .competing_variants
                    .iter()
                    .all(|variant| variant.contains("Right variant"))
        }),
        "expected duplicate competing content replacements to be preserved, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_metadata_replacements() {
    let store_root = temp_dir("merge-metadata-duplicate-non-primary-replacements");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-duplicate-non-primary-replacements".to_string(),
            title: "Merge Metadata Duplicate Non Primary Replacements".to_string(),
            language: "en".to_string(),
            timestamp: 36,
        },
    )
    .expect("document should be created");

    let left_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-duplicate-non-primary-replacements",
        &document.genesis_revision_id,
        37,
        38,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "left"
                }
            }
        ]),
    );
    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-duplicate-non-primary-replacements",
        &document.genesis_revision_id,
        39,
        40,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-duplicate-non-primary-replacements",
        &document.genesis_revision_id,
        41,
        42,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-duplicate-non-primary-replacements".to_string(),
            parents: vec![left_revision_id, right_revision_id, center_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-duplicate-non-primary-replacements".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 43,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' selected one non-primary replacement while other competing non-primary replacements remained"
        )),
        "expected competing metadata replacement reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.competing_variants == vec!["\"right\"".to_string()]
        }),
        "expected selected metadata replacement detail to retain one remaining duplicate, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacements)
                && detail.competing_variants
                    == vec!["\"right\"".to_string(), "\"right\"".to_string()]
        }),
        "expected duplicate competing metadata replacements to be preserved, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_block_is_added_from_non_primary_parent() {
    let store_root = temp_dir("merge-content-added-non-primary");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-added-non-primary".to_string(),
            title: "Merge Content Added Non Primary".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-added-non-primary",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-added-non-primary".to_string(),
            parents: vec![document.genesis_revision_id.clone(), right_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-added-non-primary".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-added", "right")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 23,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason
            .contains("block 'blk:merge-content-added' adopted a non-primary parent addition")),
        "expected added-from-parent multi-variant reason, got {summary:?}"
    );
    assert!(
        !summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added' kept the primary variant while multiple competing non-primary additions remained"
        )),
        "did not expect competing content reason with only one alternative, got {summary:?}"
    );
    let detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:merge-content-added"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::Content
        })
        .expect("expected content addition detail");
    assert_eq!(
        detail.branch_kind,
        Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_content_addition_with_competing_additions() {
    let store_root = temp_dir("merge-content-selected-addition-with-competing-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-selected-addition-with-competing-additions".to_string(),
            title: "Merge Content Selected Addition With Competing Additions".to_string(),
            language: "en".to_string(),
            timestamp: 24,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-selected-addition-with-competing-additions",
        &document.genesis_revision_id,
        25,
        26,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-choice",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-selected-addition-with-competing-additions",
        &document.genesis_revision_id,
        27,
        28,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-choice",
                    "block_type": "paragraph",
                    "content": "center",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-selected-addition-with-competing-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-selected-addition-with-competing-additions".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-added-choice", "right")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 29,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added-choice' adopted a non-primary parent addition"
        )),
        "expected selected content addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added-choice' selected one non-primary addition while other competing non-primary additions remained"
        )),
        "expected competing content addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
        }),
        "expected selected content addition detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
        }),
        "expected competing content addition detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_content_additions() {
    let store_root = temp_dir("merge-content-duplicate-non-primary-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-duplicate-non-primary-additions".to_string(),
            title: "Merge Content Duplicate Non Primary Additions".to_string(),
            language: "en".to_string(),
            timestamp: 30,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-duplicate-non-primary-additions",
        &document.genesis_revision_id,
        31,
        32,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-duplicate",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-duplicate-non-primary-additions",
        &document.genesis_revision_id,
        33,
        34,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-duplicate",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-duplicate-non-primary-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-duplicate-non-primary-additions".to_string(),
                blocks: vec![paragraph_block(
                    "blk:merge-content-added-duplicate",
                    "right",
                )],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 35,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added-duplicate' selected one non-primary addition while other competing non-primary additions remained"
        )),
        "expected duplicate content addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-duplicate"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
                && detail.competing_variants.len() == 1
                && detail.competing_variants[0].contains("right")
        }),
        "expected selected duplicate content addition detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-duplicate"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
                && detail
                    .competing_variants
                    .iter()
                    .all(|variant| variant.contains("right"))
        }),
        "expected duplicate competing content additions to be preserved, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_block_keeps_primary_absence_over_non_primary_addition(
) {
    let store_root = temp_dir("merge-content-keep-primary-absence");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-keep-primary-absence".to_string(),
            title: "Merge Content Keep Primary Absence".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-absence",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-keep-primary-absence".to_string(),
            parents: vec![document.genesis_revision_id.clone(), right_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-keep-primary-absence".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 23,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 0);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added' kept the primary absence over a competing non-primary addition"
        )),
        "expected keep-primary content reason, got {summary:?}"
    );
    let detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "blk:merge-content-added"
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.variant_kind == MergeReasonVariantKind::Content
        })
        .expect("expected keep-primary content detail");
    assert_eq!(
        detail.branch_kind,
        Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
    );

    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let ops = patch_value["ops"]
        .as_array()
        .expect("generated patch ops should be an array");
    assert!(ops.is_empty(), "expected zero-op merge patch, got {ops:?}");

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_and_multiple_competing_content_additions() {
    let store_root = temp_dir("merge-content-keep-primary-multiple-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-keep-primary-multiple-additions".to_string(),
            title: "Merge Content Keep Primary Multiple Additions".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-multiple-additions",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-multiple-additions",
        &document.genesis_revision_id,
        23,
        24,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added",
                    "block_type": "paragraph",
                    "content": "center",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-keep-primary-multiple-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-keep-primary-multiple-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 25,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added' kept the primary absence over a competing non-primary addition"
        )),
        "expected keep-primary content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-added' kept the primary variant while multiple competing non-primary additions remained"
        )),
        "expected competing content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
                && detail.competing_variants.len() == 2
        }),
        "expected keep-primary content detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
        }),
        "expected multiple-competing content detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_content_additions_when_keeping_primary_absence()
{
    let store_root = temp_dir("merge-content-keep-primary-duplicate-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-keep-primary-duplicate-additions".to_string(),
            title: "Merge Content Keep Primary Duplicate Additions".to_string(),
            language: "en".to_string(),
            timestamp: 36,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-duplicate-additions",
        &document.genesis_revision_id,
        37,
        38,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-keep-duplicate",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-duplicate-additions",
        &document.genesis_revision_id,
        39,
        40,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-added-keep-duplicate",
                    "block_type": "paragraph",
                    "content": "right",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-keep-primary-duplicate-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-keep-primary-duplicate-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 41,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-keep-duplicate"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
                && detail.competing_variants.len() == 2
                && detail
                    .competing_variants
                    .iter()
                    .all(|variant| variant.contains("right"))
        }),
        "expected keep-primary duplicate content additions detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-added-keep-duplicate"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
                && detail
                    .competing_variants
                    .iter()
                    .all(|variant| variant.contains("right"))
        }),
        "expected multiple competing duplicate content additions detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_metadata_key_is_added_from_non_primary_parent() {
    let store_root = temp_dir("merge-metadata-added-non-primary");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-added-non-primary".to_string(),
            title: "Merge Metadata Added Non Primary".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-added-non-primary",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-added-non-primary".to_string(),
            parents: vec![document.genesis_revision_id.clone(), right_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-added-non-primary".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 23,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason
                .contains("metadata key 'topic' adopted a non-primary parent addition")),
        "expected metadata added-from-parent multi-variant reason, got {summary:?}"
    );
    assert!(
        !summary.merge_reasons.iter().any(|reason| reason
            .contains("metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained")),
        "did not expect competing metadata reason with only one alternative, got {summary:?}"
    );
    let detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "topic"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected metadata addition detail");
    assert_eq!(
        detail.branch_kind,
        Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_metadata_additions_when_keeping_primary_absence()
{
    let store_root = temp_dir("merge-metadata-keep-primary-duplicate-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-duplicate-additions".to_string(),
            title: "Merge Metadata Keep Primary Duplicate Additions".to_string(),
            language: "en".to_string(),
            timestamp: 36,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-duplicate-additions",
        &document.genesis_revision_id,
        37,
        38,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-duplicate-additions",
        &document.genesis_revision_id,
        39,
        40,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-duplicate-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-duplicate-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 41,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
                && detail.competing_variants
                    == vec!["\"right\"".to_string(), "\"right\"".to_string()]
        }),
        "expected keep-primary duplicate metadata additions detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants == vec!["\"right\"".to_string(), "\"right\"".to_string()]
        }),
        "expected multiple competing duplicate metadata additions detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_metadata_addition_with_competing_additions() {
    let store_root = temp_dir("merge-metadata-selected-addition-with-competing-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-selected-addition-with-competing-additions".to_string(),
            title: "Merge Metadata Selected Addition With Competing Additions".to_string(),
            language: "en".to_string(),
            timestamp: 24,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-selected-addition-with-competing-additions",
        &document.genesis_revision_id,
        25,
        26,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-selected-addition-with-competing-additions",
        &document.genesis_revision_id,
        27,
        28,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "center"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-selected-addition-with-competing-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-selected-addition-with-competing-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 29,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason
                .contains("metadata key 'topic' adopted a non-primary parent addition")),
        "expected selected metadata addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' selected one non-primary addition while other competing non-primary additions remained"
        )),
        "expected competing metadata addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
        }),
        "expected selected metadata addition detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
        }),
        "expected competing metadata addition detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_duplicate_non_primary_metadata_additions() {
    let store_root = temp_dir("merge-metadata-duplicate-non-primary-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-duplicate-non-primary-additions".to_string(),
            title: "Merge Metadata Duplicate Non Primary Additions".to_string(),
            language: "en".to_string(),
            timestamp: 30,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-duplicate-non-primary-additions",
        &document.genesis_revision_id,
        31,
        32,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-duplicate-non-primary-additions",
        &document.genesis_revision_id,
        33,
        34,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-duplicate-non-primary-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-duplicate-non-primary-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 35,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' selected one non-primary addition while other competing non-primary additions remained"
        )),
        "expected duplicate metadata addition reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
                && detail.competing_variants == vec!["\"right\"".to_string()]
        }),
        "expected selected duplicate metadata addition detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants
                    == vec!["\"right\"".to_string(), "\"right\"".to_string()]
        }),
        "expected duplicate competing metadata additions to be preserved, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_and_multiple_competing_metadata_additions() {
    let store_root = temp_dir("merge-metadata-keep-primary-multiple-additions");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-multiple-additions".to_string(),
            title: "Merge Metadata Keep Primary Multiple Additions".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-multiple-additions",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let center_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-multiple-additions",
        &document.genesis_revision_id,
        23,
        24,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "center"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-multiple-additions".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                right_revision_id,
                center_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-multiple-additions".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 25,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary absence over a competing non-primary addition"
        )),
        "expected keep-primary metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason
            .contains("metadata key 'topic' kept the primary variant while multiple competing non-primary additions remained")),
        "expected competing metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
                && detail.competing_variants.len() == 2
        }),
        "expected keep-primary metadata detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryAdditions)
                && detail.competing_variants.len() == 2
        }),
        "expected multiple-competing metadata detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_multi_variant_when_metadata_keeps_primary_over_non_primary_addition() {
    let store_root = temp_dir("merge-metadata-keep-primary-over-addition");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-addition".to_string(),
            title: "Merge Metadata Keep Primary Over Addition".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let right_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-addition",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-addition".to_string(),
            parents: vec![document.genesis_revision_id.clone(), right_revision_id],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-over-addition".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 23,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 0);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary absence over a competing non-primary addition"
        )),
        "expected metadata keep-primary multi-variant reason, got {summary:?}"
    );
    let detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "topic"
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected metadata keep-primary detail");
    assert_eq!(
        detail.branch_kind,
        Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
    );

    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let ops = patch_value["ops"]
        .as_array()
        .expect("generated patch ops should be an array");
    assert!(ops.is_empty(), "expected zero-op merge patch, got {ops:?}");

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_preserves_distinct_reasons_for_mixed_metadata_keys() {
    let store_root = temp_dir("merge-metadata-mixed-keys");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-mixed-keys".to_string(),
            title: "Merge Metadata Mixed Keys".to_string(),
            language: "en".to_string(),
            timestamp: 20,
        },
    )
    .expect("document should be created");

    let topic_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-mixed-keys",
        &document.genesis_revision_id,
        21,
        22,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );
    let priority_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-mixed-keys",
        &document.genesis_revision_id,
        23,
        24,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "priority": "high"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-mixed-keys".to_string(),
            parents: vec![
                document.genesis_revision_id.clone(),
                topic_revision_id,
                priority_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-mixed-keys".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 25,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason
                .contains("metadata key 'topic' adopted a non-primary parent addition")),
        "expected topic selection reason, got {summary:?}"
    );
    assert!(
        summary
            .merge_reasons
            .iter()
            .any(|reason| reason.contains(
                "metadata key 'priority' kept the primary absence over a competing non-primary addition"
            )),
        "expected priority keep-primary reason, got {summary:?}"
    );
    let topic_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "topic"
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected topic detail");
    assert_eq!(
        topic_detail.branch_kind,
        Some(MergeReasonBranchKind::AdoptedNonPrimaryAddition)
    );
    let priority_detail = summary
        .merge_reason_details
        .iter()
        .find(|detail| {
            detail.subject_id == "priority"
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.variant_kind == MergeReasonVariantKind::Metadata
        })
        .expect("expected priority detail");
    assert_eq!(
        priority_detail.branch_kind,
        Some(MergeReasonBranchKind::KeptPrimaryAbsenceOverNonPrimaryAddition)
    );

    let patch_value = load_stored_object_value(&store_root, &summary.patch_id)
        .expect("generated merge patch should be stored");
    let ops = patch_value["ops"]
        .as_array()
        .expect("generated patch ops should be an array");
    assert_eq!(ops.len(), 1);
    assert_eq!(ops[0]["op"], "set_metadata");
    assert_eq!(ops[0]["metadata"]["topic"], "right");
    assert!(ops[0]["metadata"].get("priority").is_none());

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_non_primary_content_removal_as_distinct_branch() {
    let store_root = temp_dir("merge-content-remove-non-primary");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-remove-non-primary".to_string(),
            title: "Merge Content Remove Non Primary".to_string(),
            language: "en".to_string(),
            timestamp: 30,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-remove-non-primary",
        &document.genesis_revision_id,
        31,
        32,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-remove",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-remove-non-primary".to_string(),
            parents: vec![base_revision_id, document.genesis_revision_id.clone()],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-remove-non-primary".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 33,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert_eq!(summary.patch_op_count, 1);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason
            .contains("block 'blk:merge-content-remove' adopted a non-primary parent removal")),
        "expected removal-specific content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-remove"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryRemoval)
        }),
        "expected removal-specific content detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_content_removal_with_competing_removals() {
    let store_root = temp_dir("merge-content-select-removal-with-competing-removals");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-select-removal-with-competing-removals".to_string(),
            title: "Merge Content Select Removal With Competing Removals".to_string(),
            language: "en".to_string(),
            timestamp: 34,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-removal-with-competing-removals",
        &document.genesis_revision_id,
        35,
        36,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-remove-choice",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let unrelated_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-removal-with-competing-removals",
        &document.genesis_revision_id,
        37,
        38,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-unrelated",
                    "block_type": "paragraph",
                    "content": "Unrelated",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-select-removal-with-competing-removals".to_string(),
            parents: vec![
                base_revision_id,
                document.genesis_revision_id.clone(),
                unrelated_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-select-removal-with-competing-removals".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-unrelated", "Unrelated")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 39,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-remove-choice' adopted a non-primary parent removal"
        )),
        "expected selected content removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-remove-choice' selected one non-primary removal while other competing non-primary removals remained"
        )),
        "expected competing content removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-remove-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind == Some(MergeReasonBranchKind::AdoptedNonPrimaryRemoval)
                && detail.competing_variants == vec!["<absent>".to_string()]
        }),
        "expected selected content removal detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-remove-choice"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryRemovals)
                && detail.competing_variants == vec!["<absent>".to_string(), "<absent>".to_string()]
        }),
        "expected competing content removal detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_content_over_multiple_removals() {
    let store_root = temp_dir("merge-content-keep-primary-over-multiple-removals");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-keep-primary-over-multiple-removals".to_string(),
            title: "Merge Content Keep Primary Over Multiple Removals".to_string(),
            language: "en".to_string(),
            timestamp: 40,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-over-multiple-removals",
        &document.genesis_revision_id,
        41,
        42,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-remove-keep",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let unrelated_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-over-multiple-removals",
        &document.genesis_revision_id,
        43,
        44,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-other",
                    "block_type": "paragraph",
                    "content": "Other",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-keep-primary-over-multiple-removals".to_string(),
            parents: vec![
                base_revision_id,
                document.genesis_revision_id.clone(),
                unrelated_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-keep-primary-over-multiple-removals".to_string(),
                blocks: vec![
                    paragraph_block("blk:merge-content-remove-keep", "Base"),
                    paragraph_block("blk:merge-content-other", "Other"),
                ],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 45,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-remove-keep' kept the primary parent variant over a competing non-primary removal"
        )),
        "expected keep-primary content removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-remove-keep' kept the primary variant while multiple competing non-primary removals remained"
        )),
        "expected competing content removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-remove-keep"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryVariantOverNonPrimaryRemoval)
                && detail.competing_variants == vec!["<absent>".to_string(), "<absent>".to_string()]
        }),
        "expected keep-primary content removal detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-remove-keep"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryRemovals)
                && detail.competing_variants == vec!["<absent>".to_string(), "<absent>".to_string()]
        }),
        "expected multiple competing content removals detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_mixed_content_replacement_and_removal_competition() {
    let store_root = temp_dir("merge-content-mixed-replace-remove");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-mixed-replace-remove".to_string(),
            title: "Merge Content Mixed Replace Remove".to_string(),
            language: "en".to_string(),
            timestamp: 40,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-mixed-replace-remove",
        &document.genesis_revision_id,
        41,
        42,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-mixed",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let replacement_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-mixed-replace-remove",
        &base_revision_id,
        43,
        44,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-mixed",
                "new_content": "Right"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-mixed-replace-remove".to_string(),
            parents: vec![
                base_revision_id,
                replacement_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-mixed-replace-remove".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-mixed", "Base")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 45,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-mixed' kept the primary parent variant over mixed competing non-primary alternatives"
        )),
        "expected mixed keep-primary content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-mixed' kept the primary variant while multiple competing non-primary replacements and removals remained"
        )),
        "expected mixed multiple-competing content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-mixed"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::KeptPrimaryVariantOverMixedNonPrimaryAlternatives,
                    )
        }),
        "expected mixed keep-primary content detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-mixed"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingMixedNonPrimaryAlternatives)
        }),
        "expected mixed multiple-competing content detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_replacement_with_competing_removal_as_distinct_branch() {
    let store_root = temp_dir("merge-content-select-replace-with-removal");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-select-replace-with-removal".to_string(),
            title: "Merge Content Select Replace With Removal".to_string(),
            language: "en".to_string(),
            timestamp: 46,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-replace-with-removal",
        &document.genesis_revision_id,
        47,
        48,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-select",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let replacement_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-replace-with-removal",
        &base_revision_id,
        49,
        50,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-select",
                "new_content": "Right"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-select-replace-with-removal".to_string(),
            parents: vec![
                base_revision_id,
                replacement_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-select-replace-with-removal".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-select", "Right")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 51,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-select' adopted a non-primary parent replacement while a competing non-primary removal remained"
        )),
        "expected mixed selected replacement reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-select"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::AdoptedNonPrimaryReplacementWhileCompetingRemovalRemains,
                    )
        }),
        "expected mixed selected replacement detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-select"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingMixedNonPrimaryAlternatives)
        }),
        "expected mixed selected competing content detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_metadata_removal_competition_as_distinct_branch() {
    let store_root = temp_dir("merge-metadata-keep-primary-over-removal");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-removal".to_string(),
            title: "Merge Metadata Keep Primary Over Removal".to_string(),
            language: "en".to_string(),
            timestamp: 50,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-removal",
        &document.genesis_revision_id,
        51,
        52,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-removal".to_string(),
            parents: vec![base_revision_id, document.genesis_revision_id.clone()],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-over-removal".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("base".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 53,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary parent variant over a competing non-primary removal"
        )),
        "expected removal-specific metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryVariantOverNonPrimaryRemoval)
        }),
        "expected removal-specific metadata detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_metadata_over_multiple_removals() {
    let store_root = temp_dir("merge-metadata-keep-primary-over-multiple-removals");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-multiple-removals".to_string(),
            title: "Merge Metadata Keep Primary Over Multiple Removals".to_string(),
            language: "en".to_string(),
            timestamp: 54,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-multiple-removals",
        &document.genesis_revision_id,
        55,
        56,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );
    let unrelated_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-multiple-removals",
        &document.genesis_revision_id,
        57,
        58,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "priority": "high"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-multiple-removals".to_string(),
            parents: vec![
                base_revision_id,
                document.genesis_revision_id.clone(),
                unrelated_revision_id,
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-over-multiple-removals".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([
                    ("topic".to_string(), Value::String("base".to_string())),
                    ("priority".to_string(), Value::String("high".to_string())),
                ]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 59,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary parent variant over a competing non-primary removal"
        )),
        "expected keep-primary metadata removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary variant while multiple competing non-primary removals remained"
        )),
        "expected competing metadata removal reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::KeptPrimaryVariantOverNonPrimaryRemoval)
                && detail.competing_variants == vec!["<absent>".to_string(), "<absent>".to_string()]
        }),
        "expected keep-primary metadata removal detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingNonPrimaryRemovals)
                && detail.competing_variants == vec!["<absent>".to_string(), "<absent>".to_string()]
        }),
        "expected multiple competing metadata removals detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_metadata_replacement_with_competing_removal_as_distinct_branch()
{
    let store_root = temp_dir("merge-metadata-select-replace-with-removal");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-select-replace-with-removal".to_string(),
            title: "Merge Metadata Select Replace With Removal".to_string(),
            language: "en".to_string(),
            timestamp: 54,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-select-replace-with-removal",
        &document.genesis_revision_id,
        55,
        56,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );
    let replacement_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-select-replace-with-removal",
        &base_revision_id,
        57,
        58,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-select-replace-with-removal".to_string(),
            parents: vec![
                base_revision_id,
                replacement_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-select-replace-with-removal".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 59,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' adopted a non-primary parent replacement while a competing non-primary removal remained"
        )),
        "expected mixed selected metadata replacement reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::AdoptedNonPrimaryReplacementWhileCompetingRemovalRemains,
                    )
        }),
        "expected mixed selected metadata replacement detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(MergeReasonBranchKind::MultipleCompetingMixedNonPrimaryAlternatives)
        }),
        "expected mixed selected competing metadata detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_content_replacement_with_multiple_replacements_and_removal() {
    let store_root = temp_dir("merge-content-select-replace-with-many-variants");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-select-replace-with-many-variants".to_string(),
            title: "Merge Content Select Replace With Many Variants".to_string(),
            language: "en".to_string(),
            timestamp: 60,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-replace-with-many-variants",
        &document.genesis_revision_id,
        61,
        62,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-select-many",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let replacement_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-replace-with-many-variants",
        &base_revision_id,
        63,
        64,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-select-many",
                "new_content": "Right A"
            }
        ]),
    );
    let replacement_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-select-replace-with-many-variants",
        &base_revision_id,
        65,
        66,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-select-many",
                "new_content": "Right B"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-select-replace-with-many-variants".to_string(),
            parents: vec![
                base_revision_id,
                replacement_a_revision_id,
                replacement_b_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-select-replace-with-many-variants".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-select-many", "Right A")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 67,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-select-many' adopted a non-primary parent replacement while competing non-primary replacements and a removal remained"
        )),
        "expected richer selected content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-select-many' selected one non-primary alternative while multiple competing non-primary replacements and removals remained"
        )),
        "expected richer competing content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-select-many"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::AdoptedNonPrimaryReplacementWhileCompetingReplacementsAndRemovalRemain,
                    )
        }),
        "expected richer selected content branch detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-select-many"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer competing content branch detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_content_over_multiple_replacements_and_removals() {
    let store_root = temp_dir("merge-content-keep-primary-over-many-variants");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-content-keep-primary-over-many-variants".to_string(),
            title: "Merge Content Keep Primary Over Many Variants".to_string(),
            language: "en".to_string(),
            timestamp: 68,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-over-many-variants",
        &document.genesis_revision_id,
        69,
        70,
        json!([
            {
                "op": "insert_block",
                "new_block": {
                    "block_id": "blk:merge-content-keep-many",
                    "block_type": "paragraph",
                    "content": "Base",
                    "attrs": {},
                    "children": []
                }
            }
        ]),
    );
    let replacement_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-over-many-variants",
        &base_revision_id,
        71,
        72,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-keep-many",
                "new_content": "Right A"
            }
        ]),
    );
    let replacement_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-content-keep-primary-over-many-variants",
        &base_revision_id,
        73,
        74,
        json!([
            {
                "op": "replace_block",
                "block_id": "blk:merge-content-keep-many",
                "new_content": "Right B"
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-content-keep-primary-over-many-variants".to_string(),
            parents: vec![
                base_revision_id,
                replacement_a_revision_id,
                replacement_b_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-content-keep-primary-over-many-variants".to_string(),
                blocks: vec![paragraph_block("blk:merge-content-keep-many", "Base")],
                metadata: serde_json::Map::new(),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 75,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-keep-many' kept the primary parent variant over multiple competing non-primary replacements and removals"
        )),
        "expected richer kept-primary content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "block 'blk:merge-content-keep-many' kept the primary variant while multiple competing non-primary replacements and removals remained"
        )),
        "expected richer competing kept-primary content reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-keep-many"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::KeptPrimaryVariantOverMultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer kept-primary content branch detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "blk:merge-content-keep-many"
                && detail.variant_kind == MergeReasonVariantKind::Content
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer multiple competing kept-primary content branch detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_selected_metadata_replacement_with_multiple_replacements_and_removal() {
    let store_root = temp_dir("merge-metadata-select-replace-with-many-variants");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-select-replace-with-many-variants".to_string(),
            title: "Merge Metadata Select Replace With Many Variants".to_string(),
            language: "en".to_string(),
            timestamp: 76,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-select-replace-with-many-variants",
        &document.genesis_revision_id,
        77,
        78,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );
    let replacement_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-select-replace-with-many-variants",
        &base_revision_id,
        79,
        80,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right-a"
                }
            }
        ]),
    );
    let replacement_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-select-replace-with-many-variants",
        &base_revision_id,
        81,
        82,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right-b"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-select-replace-with-many-variants".to_string(),
            parents: vec![
                base_revision_id,
                replacement_a_revision_id,
                replacement_b_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-select-replace-with-many-variants".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("right-a".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 83,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' adopted a non-primary parent replacement while competing non-primary replacements and a removal remained"
        )),
        "expected richer selected metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' selected one non-primary alternative while multiple competing non-primary replacements and removals remained"
        )),
        "expected richer competing metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind == MergeReasonKind::SelectedNonPrimaryParentVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::AdoptedNonPrimaryReplacementWhileCompetingReplacementsAndRemovalRemain,
                    )
        }),
        "expected richer selected metadata branch detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterSelectedVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer competing metadata branch detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}

#[test]
fn merge_authoring_reports_kept_primary_metadata_over_multiple_replacements_and_removals() {
    let store_root = temp_dir("merge-metadata-keep-primary-over-many-variants");
    let signing_key = signing_key();
    let document = create_document_in_store(
        &store_root,
        &signing_key,
        &DocumentCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-many-variants".to_string(),
            title: "Merge Metadata Keep Primary Over Many Variants".to_string(),
            language: "en".to_string(),
            timestamp: 84,
        },
    )
    .expect("document should be created");

    let base_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-many-variants",
        &document.genesis_revision_id,
        85,
        86,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "base"
                }
            }
        ]),
    );
    let replacement_a_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-many-variants",
        &base_revision_id,
        87,
        88,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right-a"
                }
            }
        ]),
    );
    let replacement_b_revision_id = commit_ops_revision(
        &store_root,
        &signing_key,
        "doc:merge-metadata-keep-primary-over-many-variants",
        &base_revision_id,
        89,
        90,
        json!([
            {
                "op": "set_metadata",
                "metadata": {
                    "topic": "right-b"
                }
            }
        ]),
    );

    let summary = create_merge_revision_in_store(
        &store_root,
        &signing_key,
        &MergeRevisionCreateParams {
            doc_id: "doc:merge-metadata-keep-primary-over-many-variants".to_string(),
            parents: vec![
                base_revision_id,
                replacement_a_revision_id,
                replacement_b_revision_id,
                document.genesis_revision_id.clone(),
            ],
            resolved_state: crate::replay::DocumentState {
                doc_id: "doc:merge-metadata-keep-primary-over-many-variants".to_string(),
                blocks: Vec::new(),
                metadata: serde_json::Map::from_iter([(
                    "topic".to_string(),
                    Value::String("base".to_string()),
                )]),
            },
            merge_strategy: "semantic-block-merge".to_string(),
            timestamp: 91,
        },
    )
    .expect("merge revision should be created");

    assert_eq!(summary.merge_outcome, MergeOutcome::MultiVariant);
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary parent variant over multiple competing non-primary replacements and removals"
        )),
        "expected richer kept-primary metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reasons.iter().any(|reason| reason.contains(
            "metadata key 'topic' kept the primary variant while multiple competing non-primary replacements and removals remained"
        )),
        "expected richer competing kept-primary metadata reason, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::KeptPrimaryParentVariantOverCompetingNonPrimaryAlternative
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::KeptPrimaryVariantOverMultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer kept-primary metadata branch detail, got {summary:?}"
    );
    assert!(
        summary.merge_reason_details.iter().any(|detail| {
            detail.subject_id == "topic"
                && detail.variant_kind == MergeReasonVariantKind::Metadata
                && detail.reason_kind
                    == MergeReasonKind::MultipleCompetingAlternativesRemainAfterKeepingPrimaryVariant
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::MultipleCompetingNonPrimaryReplacementsAndRemovals,
                    )
        }),
        "expected richer multiple competing kept-primary metadata branch detail, got {summary:?}"
    );

    let _ = fs::remove_dir_all(store_root);
}
