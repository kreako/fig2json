pub mod decoder;
pub mod transformations;
pub mod tree;

// Re-export commonly used items
pub use decoder::decode_fig_to_json;
pub use transformations::{
    remove_background_properties, remove_border_weights,
    remove_constraint_properties, remove_corner_smoothing, remove_default_blend_mode,
    remove_default_opacity, remove_default_rotation, remove_default_text_properties,
    remove_default_visible, remove_derived_text_layout_size,
    remove_detached_symbol_id, remove_document_properties, remove_edit_info_fields,
    remove_empty_derived_text_data, remove_empty_font_postscript, remove_empty_objects,
    remove_empty_paint_arrays, remove_export_settings,
    remove_frame_properties, remove_geometry_fields, remove_guid_fields, remove_guid_paths,
    remove_image_metadata_fields, remove_internal_only_nodes, remove_invisible_paints,
    remove_layout_aids, remove_overridden_symbol_id, remove_phase_fields, remove_plugin_data,
    remove_rectangle_corner_radii_independent, remove_redundant_corner_radii,
    remove_redundant_padding, remove_root_blobs, remove_root_metadata,
    remove_scroll_resize_properties, remove_stack_align_items, remove_stack_child_properties,
    remove_stack_sizing_properties, remove_stroke_properties, remove_style_ids,
    remove_symbol_id_fields, remove_text_glyphs,
    remove_text_layout_fields, remove_default_text_line_properties, remove_text_metadata_fields, remove_type,
    remove_user_facing_versions, remove_visible_only_objects, simplify_enums, simplify_text_properties,
    transform_colors_to_css, transform_image_hashes, transform_matrix_to_css,
};
pub use tree::build_tree;
