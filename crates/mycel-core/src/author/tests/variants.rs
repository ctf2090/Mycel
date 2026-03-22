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
        vec![
            "\"center\"".to_string(),
            "\"right\"".to_string(),
        ],
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
                && detail.branch_kind
                    == Some(
                        MergeReasonBranchKind::AdoptedNonPrimaryRemovalWhileCompetingReplacementRemains,
                    )
        }),
        "expected removal-specific content detail, got {summary:?}"
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
