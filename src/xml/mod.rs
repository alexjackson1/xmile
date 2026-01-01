// An XMILE view contains XMILE display objects. A view can be thought of as a page, or a screen of a model’s stock and flow diagram, or its interface. An XMILE view has an OPTIONAL type parameter to notify users of the model as to whether or not the view is a part of the stock and flow diagram, or if the view is a part of the user interface. When a type is not specified on a view it is RECOMMENDED that the view be classified as a “stock_flow” view.  Other acceptable view types are popup (for popup windows) or vendor specific types.

// Views which have the type “stock_flow” are assumed to contain display objects which make up the stock and flow diagram. Typical objects appearing in a “stock_flow” view are stocks, flows, auxiliaries, aliases, and connectors. “stock_flow” views can also contain objects which are normally associated with an interface like sliders, graphs, and tables.

// Views which have the optional type “interface” are assumed to contain display objects which make up the interactive learning environment or user interface for the model. Any display object can be present in a view with the user interface type with the REQUIRED exception of the canonical representation of a model variable object (see Section 5.1.1). Said another way, model variables cannot be defined in a view marked as being a part of the user interface.

// XMILE views also contain an OPTIONAL order attribute which represents the order that views should be presented within the application. The order attribute is an integer starting from 0 and counting up. The lower the order number the earlier it appears in the list. Views with the type “stock_flow” are ordered separately from views with the type “interface”. If any view does not contain an order number it is RECOMMENDED that one is be assigned based on its position within the <views> tag.

// XMILE views are also REQUIRED to have a width and a height measured in pixels.  These properties describe the size of a view.  Views are REQUIRED to be rectangular.   Views may also have an OPTIONAL zoom specified as a double where 100 is default, 200 is 2x bigger by a factor of 2 and 50 is smaller by a factor of 2. In addition views can also have OPTIONAL attributes for storing the scroll position called scroll_x and scroll_y.  The scroll position is the origin (top left corner) of the screen in model coordinates (that is, when zoom has not been applied).  Also, views may contain an OPTIONAL background attribute that may be specified either as a color or as an external image resource using a file://url.

// In order for XMILE views to be printed each view is REQUIRED to specify its paging.  Therefore the following attributes are REQUIRED:

//     page_width<double> - The width of a printed page
//     page_height<double> - The height of a printed page
//     page_sequence<string> “row|column” – The ordering of page numbers.  With sequence type row, numbers are assigned going from left to right, then top to bottom.  With the sequence being column pages are ordered from top to bottom then left to right.
//     page_orientation<string> “landscape|portrait” – The orientation of the view on the printed page.
//     show_pages<bool> - Whether or not the software overlays page breaks and page numbers on the screen.

// In order for XMILE views to be more easily navigated views are REQUIRED to specify:

//     home_page<int> default: 0- The index of the printed page which is shown when any link with the home_page target is executed
//     home_view<bool> default: false – A marker property which is used to determine which view is navigated to when any link with the target home_view is executed.  Only one view for each view type is allowed to be marked as the home_view.

// 5.1.1 Referencing variable objects in an XMILE view

// Any object appearing in the <variables> tag (stock, flow, auxiliary, module, and group) is RECOMMENDED to have a related <stock|flow|aux|module|group> tag in at least one of the <view> tags associated with its model in order to be XMILE compatible.  A <stock|flow|aux|module|group> tag is linked to its associated model equation object through the use of its REQUIRED “name” attribute (see sample XMILE in beginning of chapter 5). A <stock|flow|aux|module> (note: not group) tag representing a model variable MUST NOT appear more than once in a single <view> tag. A <stock|flow|aux|module> (note: not group) tag representing a model variable may appear in separate <view> tags, but support of this feature is OPTIONAL. It is RECOMMENDED that in the case where this feature is not supported the lowest order view (or first view encountered is order is not specified) containing a <stock|flow|aux|module> tag representing a model variable is treated as the canonical display for that object and that any other encountered <stock|flow|aux|module> tag in any other <view> tag associated with that model representing the same model variable be treated as an alias (described in section 6.1.7).
// 5.1.2 XMILE view assumptions and attributes

// All visual objects contained within an XMILE <view> are laid out on a 2D Cartesian coordinate space measured in pixels, where 0,0 is the top left of the screen and height runs down while width runs right. An example coordinate space map looks like:

// All display objects contained within an XMILE file MUST have the following attributes:

//     Position:  x="<double>", y="<double>"
//     Size:  arbitrary rectangle or, for specific objects listed below, a <shape> tag
//                 width="<double>", height="<double>"

// A <shape> tag is specified as a child tag of other tags that are themselves child of a <view>.  Specifically, shape tags allow stock, auxiliary, module, or alias objects to be represented using a different symbol then the RECOMMENDED; rectangle for a stock, circle for an auxiliary and rounded rectangle for a module. It is OPTIONAL for these four object types to specify a <shape> tag to change their representation from the defaults specified above to any valid value for a <shape> tag described below with the following REQUIRED exceptions:

//     A stock MUST NOT be represented using a circle.
//     An auxiliary or flow MUST NOT be represented using a rectangle except if the equation contains a function or macro which contains a stock.

// Shape tags contain a REQUIRED type attribute which describes the shape of the object owning the tag. Valid type values are: rectangle, circle, and name_only. Shapes of type rectangle have two REQUIRED attributes, width and height, and an OPTIONAL attribute, the corner radius, all specified as doubles in pixels. Shapes of type circle contain one REQUIRED attribute:  radius. Shapes of type name_only are special:  they contain two OPTIONAL attributes width and height both measured in pixels and represented using a double. The name_only shape specifies that the object shall be represented by its name plate only. The optional width and height attributes are used for line wrapping hints.  These hints are only suggestions and may be ignored without consequence.

// The position referred to by the x and y attributes refers to the center of the object when using a <shape> tag. When using an arbitrary size, the x and y attributes refer to the top left corner of the object. All locations and sizes of XMILE objects are REQUIRED to be represented using double precision numbers.
// 5.1.3 Referring to specific XMILE display objects

// Display objects do not have names or any other way to specifically refer to individual objects. Therefore any display object which is referred to anywhere else in the XMILE file MUST provide a uid="<int>" attribute. This attribute is a unique linearly increasing integer which gives each display object a way to be referred to specifically while reading in an XMILE file. UIDs are NOT REQUIRED to be stable across successive reads and writes. Objects requiring a uid are listed in Chapter 6 of this specification. UIDs MUST be unique per XMILE model.

pub mod schema;
