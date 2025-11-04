pub mod decoder;
pub mod transformations;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use transformations::{
    remove_background_properties, remove_border_weights, remove_default_blend_mode,
    remove_default_text_properties, remove_derived_text_layout_size, remove_document_properties,
    remove_edit_info_fields, remove_empty_font_postscript, remove_frame_properties,
    remove_geometry_fields, remove_guid_fields, remove_image_metadata_fields,
    remove_internal_only_nodes, remove_phase_fields, remove_root_blobs, remove_stroke_properties,
    remove_text_data_fields, remove_text_glyphs, remove_text_layout_fields,
    remove_text_metadata_fields, simplify_enums, transform_colors_to_css, transform_image_hashes,
    transform_matrix_to_css,
};
pub use tree::build_tree;
