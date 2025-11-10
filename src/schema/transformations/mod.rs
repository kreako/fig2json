/// Transformation passes for the final JSON output
///
/// This module contains various transformation passes that are applied to the
/// JSON document after initial parsing and blob substitution:
///
/// - `image_hash`: Convert image hash arrays to filename strings
/// - `blobs_removal`: Remove the root-level blobs array from final output
/// - `matrix_to_css`: Convert 2D affine transformation matrices to CSS positioning properties
/// - `color_to_css`: Convert RGBA color objects to CSS hex color strings
/// - `text_glyphs_removal`: Remove glyph vector data from text objects
/// - `guid_removal`: Remove internal Figma guid identifiers
/// - `edit_info_removal`: Remove version control edit info metadata
/// - `phase_removal`: Remove Figma internal phase state
/// - `geometry_removal`: Remove detailed geometry path commands
/// - `text_layout_removal`: Remove detailed text layout data
/// - `text_metadata_removal`: Remove text configuration metadata
/// - `text_line_defaults_removal`: Remove default text line properties from lines arrays
/// - `stroke_properties_removal`: Remove CSS-incompatible stroke properties
/// - `frame_properties_removal`: Remove frame-specific metadata
/// - `image_metadata_removal`: Remove image metadata fields
/// - `document_properties_removal`: Remove document-level properties
/// - `enum_simplification`: Simplify verbose enum objects to simple strings
/// - `default_text_properties_removal`: Remove default text property values
/// - `empty_font_postscript_removal`: Remove empty postscript from fontName
/// - `border_weights_removal`: Remove individual border weight fields
/// - `default_blend_mode_removal`: Remove default blendMode values
/// - `background_properties_removal`: Remove background metadata fields
/// - `internal_only_nodes_removal`: Filter out internal-only nodes
/// - `derived_text_layout_size_removal`: Remove redundant layoutSize from derivedTextData
/// - `empty_derived_text_data_removal`: Remove empty derivedTextData objects
/// - `empty_objects_removal`: Remove empty objects {} from the JSON tree
/// - `default_opacity_removal`: Remove default opacity values (1.0)
/// - `default_visible_removal`: Remove default visible values (true)
/// - `default_rotation_removal`: Remove default rotation values (0.0)
/// - `root_metadata_removal`: Remove root-level version and fileType fields
/// - `guid_path_removal`: Remove internal Figma guidPath references
/// - `user_facing_version_removal`: Remove Figma version strings
/// - `style_id_removal`: Remove Figma shared style references
/// - `export_settings_removal`: Remove asset export configurations
/// - `plugin_data_removal`: Remove Figma plugin storage data
/// - `rectangle_corner_radii_independent_removal`: Remove corner radii independent flag
/// - `constraint_properties_removal`: Remove Figma auto-layout constraint properties
/// - `scroll_resize_properties_removal`: Remove Figma scroll and resize behavior properties
/// - `layout_aids_removal`: Remove design-time layout aids (guides, layoutGrids)
/// - `detached_symbol_id_removal`: Remove Figma component instance metadata
/// - `redundant_corner_radii_removal`: Remove individual corner radius fields when general cornerRadius exists
/// - `corner_smoothing_removal`: Remove Figma's corner smoothing property
/// - `invisible_paints_removal`: Remove invisible paints from fillPaints and strokePaints arrays
/// - `stack_child_properties_removal`: Remove Figma auto-layout child properties (stackChildAlignSelf, stackChildPrimaryGrow)
/// - `redundant_padding_removal`: Remove redundant padding properties when general axis-based padding exists
/// - `stack_sizing_properties_removal`: Remove Figma auto-layout sizing properties (stackCounterSizing, stackPrimarySizing)
/// - `stack_align_items_removal`: Remove Figma auto-layout alignment properties (stackCounterAlignItems, stackPrimaryAlignItems)
/// - `text_properties_simplification`: Simplify verbose letterSpacing/lineHeight structures to CSS-ready strings
/// - `type_removal`: Remove type field from all nodes
/// - `empty_paint_arrays_removal`: Remove empty fillPaints and strokePaints arrays
/// - `overridden_symbol_id_removal`: Remove standalone overriddenSymbolID objects from arrays
/// - `symbol_id_removal`: Remove symbolID objects containing only localID and/or sessionID
pub mod background_properties_removal;
pub mod blobs_removal;
pub mod border_weights_removal;
pub mod color_to_css;
pub mod constraint_properties_removal;
pub mod corner_smoothing_removal;
pub mod default_blend_mode_removal;
pub mod default_opacity_removal;
pub mod default_rotation_removal;
pub mod default_text_properties_removal;
pub mod default_visible_removal;
pub mod derived_text_layout_size_removal;
pub mod detached_symbol_id_removal;
pub mod document_properties_removal;
pub mod empty_derived_text_data_removal;
pub mod empty_paint_arrays_removal;
pub mod edit_info_removal;
pub mod empty_font_postscript_removal;
pub mod empty_objects_removal;
pub mod enum_simplification;
pub mod export_settings_removal;
pub mod frame_properties_removal;
pub mod geometry_removal;
pub mod guid_path_removal;
pub mod guid_removal;
pub mod image_hash;
pub mod image_metadata_removal;
pub mod internal_only_nodes_removal;
pub mod invisible_paints_removal;
pub mod layout_aids_removal;
pub mod matrix_to_css;
pub mod overridden_symbol_id_removal;
pub mod phase_removal;
pub mod plugin_data_removal;
pub mod rectangle_corner_radii_independent_removal;
pub mod redundant_corner_radii_removal;
pub mod redundant_padding_removal;
pub mod root_metadata_removal;
pub mod scroll_resize_properties_removal;
pub mod stack_align_items_removal;
pub mod stack_child_properties_removal;
pub mod stack_sizing_properties_removal;
pub mod stroke_properties_removal;
pub mod style_id_removal;
pub mod symbol_id_removal;
pub mod text_glyphs_removal;
pub mod text_layout_removal;
pub mod text_line_defaults_removal;
pub mod text_metadata_removal;
pub mod text_properties_simplification;
pub mod type_removal;
pub mod user_facing_version_removal;

// Re-export commonly used functions
pub use background_properties_removal::remove_background_properties;
pub use blobs_removal::remove_root_blobs;
pub use border_weights_removal::remove_border_weights;
pub use color_to_css::transform_colors_to_css;
pub use constraint_properties_removal::remove_constraint_properties;
pub use corner_smoothing_removal::remove_corner_smoothing;
pub use default_blend_mode_removal::remove_default_blend_mode;
pub use default_opacity_removal::remove_default_opacity;
pub use default_rotation_removal::remove_default_rotation;
pub use default_text_properties_removal::remove_default_text_properties;
pub use default_visible_removal::remove_default_visible;
pub use derived_text_layout_size_removal::remove_derived_text_layout_size;
pub use detached_symbol_id_removal::remove_detached_symbol_id;
pub use document_properties_removal::remove_document_properties;
pub use empty_derived_text_data_removal::remove_empty_derived_text_data;
pub use empty_paint_arrays_removal::remove_empty_paint_arrays;
pub use edit_info_removal::remove_edit_info_fields;
pub use empty_font_postscript_removal::remove_empty_font_postscript;
pub use empty_objects_removal::remove_empty_objects;
pub use enum_simplification::simplify_enums;
pub use export_settings_removal::remove_export_settings;
pub use frame_properties_removal::remove_frame_properties;
pub use geometry_removal::remove_geometry_fields;
pub use guid_path_removal::remove_guid_paths;
pub use guid_removal::remove_guid_fields;
pub use image_hash::transform_image_hashes;
pub use image_metadata_removal::remove_image_metadata_fields;
pub use internal_only_nodes_removal::remove_internal_only_nodes;
pub use invisible_paints_removal::remove_invisible_paints;
pub use layout_aids_removal::remove_layout_aids;
pub use matrix_to_css::transform_matrix_to_css;
pub use overridden_symbol_id_removal::remove_overridden_symbol_id;
pub use phase_removal::remove_phase_fields;
pub use plugin_data_removal::remove_plugin_data;
pub use rectangle_corner_radii_independent_removal::remove_rectangle_corner_radii_independent;
pub use redundant_corner_radii_removal::remove_redundant_corner_radii;
pub use redundant_padding_removal::remove_redundant_padding;
pub use root_metadata_removal::remove_root_metadata;
pub use scroll_resize_properties_removal::remove_scroll_resize_properties;
pub use stack_align_items_removal::remove_stack_align_items;
pub use stack_child_properties_removal::remove_stack_child_properties;
pub use stack_sizing_properties_removal::remove_stack_sizing_properties;
pub use stroke_properties_removal::remove_stroke_properties;
pub use style_id_removal::remove_style_ids;
pub use symbol_id_removal::remove_symbol_id_fields;
pub use text_glyphs_removal::remove_text_glyphs;
pub use text_layout_removal::remove_text_layout_fields;
pub use text_line_defaults_removal::remove_default_text_line_properties;
pub use text_metadata_removal::remove_text_metadata_fields;
pub use text_properties_simplification::simplify_text_properties;
pub use type_removal::remove_type;
pub use user_facing_version_removal::remove_user_facing_versions;
