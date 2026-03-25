use super::*;

fn assert_content_variant_merge_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains("adopted a non-primary parent replacement")
                })
            })),
        "expected content variant multi-variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reasons"]
            .as_array()
            .is_some_and(|reasons| reasons.iter().any(|reason| {
                reason.as_str().is_some_and(|reason| {
                    reason.contains(
                        "selected one non-primary replacement while other competing non-primary replacements remained",
                    )
                })
            })),
        "expected competing content variant reason, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["branch_kind"] == "adopted-non-primary-replacement"
                    && detail["resolved_variant"]
                        .as_str()
                        .is_some_and(|variant| variant.contains("Right variant"))
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 1)
            })),
        "expected structured content variant detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_kind"] == "block"
                    && detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| variants.len() == 2)
            })),
        "expected competing content branch kind detail, got {merge_json}"
    );
    assert_eq!(merge_json["patch_op_count"], 1);
    assert_eq!(
        merge_json["parent_revision_ids"].as_array().map(Vec::len),
        Some(3)
    );
}

fn assert_duplicate_non_primary_content_replacement_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 1
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected selected duplicate content replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "blk:author-smoke-variant-001"
                    && detail["variant_kind"] == "content"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"]
                        .as_array()
                        .is_some_and(|variants| {
                            variants.len() == 2
                                && variants.iter().all(|variant| {
                                    variant.as_str().is_some_and(|variant| {
                                        variant.contains("\"content\":\"right\"")
                                    })
                                })
                        })
            })),
        "expected duplicate competing content replacements detail, got {merge_json}"
    );
}

fn assert_duplicate_non_primary_metadata_replacement_reasons(merge_json: &serde_json::Value) {
    assert_eq!(merge_json["merge_outcome"], "multi-variant");
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"] == "selected-non-primary-parent-variant"
                    && detail["competing_variants"] == json!(["\"right\""])
            })),
        "expected selected duplicate metadata replacement detail, got {merge_json}"
    );
    assert!(
        merge_json["merge_reason_details"]
            .as_array()
            .is_some_and(|details| details.iter().any(|detail| {
                detail["subject_id"] == "topic"
                    && detail["variant_kind"] == "metadata"
                    && detail["reason_kind"]
                        == "multiple-competing-alternatives-remain-after-selected-variant"
                    && detail["branch_kind"] == "multiple-competing-non-primary-replacements"
                    && detail["competing_variants"] == json!(["\"right\"", "\"right\""])
            })),
        "expected duplicate competing metadata replacements detail, got {merge_json}"
    );
}

mod additions;
mod mixed_branches;
mod replacements;
