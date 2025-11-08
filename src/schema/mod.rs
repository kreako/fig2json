pub mod decoder;
pub mod transformations;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use transformations::{
    remove_background_properties, remove_border_weights, remove_component_properties,
    remove_default_blend_mode, remove_default_opacity, remove_default_rotation,
    remove_default_text_properties, remove_default_visible, remove_derived_symbol_data,
    remove_derived_text_layout_size, remove_document_properties, remove_edit_info_fields,
    remove_empty_derived_text_data, remove_empty_font_postscript, remove_export_settings,
    remove_frame_properties, remove_geometry_fields, remove_guid_fields, remove_guid_paths,
    remove_image_metadata_fields, remove_internal_only_nodes, remove_phase_fields,
    remove_plugin_data, remove_rectangle_corner_radii_independent, remove_root_blobs,
    remove_root_metadata, remove_stroke_properties, remove_style_ids, remove_symbol_data,
    remove_text_data_fields, remove_text_glyphs, remove_text_layout_fields,
    remove_text_metadata_fields, remove_user_facing_versions, simplify_enums,
    transform_colors_to_css, transform_image_hashes, transform_matrix_to_css,
};
pub use tree::build_tree;
