use crate::font::Font;
use crate::ligature_substitution::StrLigatureSubstitution;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::mono_font::mapping::StrGlyphMapping;

/// ![mogeefont](data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAQAAAACmCAAAAAAQKw79AAAKuElEQVR4Ae3gAZAkSZIkSRKLqpm7R0REZmZmVlVVVVV3d3d3d/fMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMdHd3d3dXV1VVVVVmZkZGRIS7m5kKz0xmV3d1d3dPz8zMzMxMYuUBjDDiuRnxLzHi+THiRWfEfykq/79ReSYjnj8DRoARAEYAGBDPyQgjAIwAMOJFZwDEFUaAARBgrhAABoS5QgAYEOYKYQAEGADxAFT+f6PyAAIEgBFGgBFgwAgj7mcEGAEgAIwAYQQIA0a8aIwAEGAEmPsJMEIAGAAjDIC4nxEAAsAACDACBBjxLFT+f6PyfBgwYAQACBBGgBHPJl4wI4wwAsCIfw0jDIB4NgMCjLifAQFG/KtQ+f+NyvMhjDAChAEBIMCIK4QBAUYAGAEAwghxhXhRGPGcxAMZASDAiGcTAEaAAXGFESAMCBAGAIwAIyr/v1F5PgwYMAIEGAFgBBgBRoARL4y4wgAYEFeYK8SLwojnZp6XACPACAAjwAgQYEAACKj8/0blmcSzCSOMACPuZwQYYcQVwoh/mQAjnk08kBEvmBFXGHGFADBgxIvKiAeg8v8blWcy4tkEiCsMiGcTRoAwAMIIAGEEgBEvjBHPZsSzCXOFATACDIAQ5rkJAwDCPDdhrjAgAIwAIyr/vyH+RUa8qIwAMOKFMeK/nxGV/98Q/8GMMOKFMeK/nxFQ+f8N8f8blf/fqPz/RuX/Nyr/v1H5/43Ki8CAeH4MCDAAAsAACAMgwAAIAyDAAAgjwAgDIADACCPACCMAjAAwwggwAgwIMAACwIAAAyAADIAw9xOV/9+o/IuMACMAzBUCjAAjQIARRoABEGAECAAQAEaAeSBhxL+FEWAECDDCCDACBBhhBBgQYARA5f83KgAYEM9mQFwhns2IK4x4wcS/TPzLjHggI8wVRvzriedA5f83KhgQAOYKIcCAADAgwAgwIJ6bAfGCGBBgQAAYEGBeMCOekzDCAAjzojAgQBgQz4HK/29UI+4nHkiAEQDifkaY+5n7CTDi+RMAIO4nDIAA8/wJI14wYZ6TAfGchLlCgBEPQOX/N6owIADMFQLAgAAQL4gA87yM+Lcx4tmEEQ8kXhjx/AgjjHgeVP5/o4IAA0Lcz4C4nxH3EwaMeE4GBAgDAsCAADAg7icMiAcyYIQwIJ6TAfH8CQPigYQB8ZwMiOdA5f83xL+KEf97GPEvoPL/G+JfyQCI/yOo/P+G+P+Nyv9vVP5/o/L/G5X/QEb8L0Pl/zeqEWCEEWCEEWAEGGEEgBFGABhhrhAGxLMZEGDE82PEczPigQyI+xkBYASAEUYAGPFvROX/Nyr/DgLAAAgQRgAIA0YY8W9hxPMjrhD/Iaj8/0blRSCMMAKEEUZcYcQLIkA8kAEBYAQYEADmCgPCgBEABgQYAUYAGAFgBIC5QoABAQYEgAFhQIABcRmV/9+ovEiEEc/LPCdhxAsmwAgQRoAwwggDRhgQRgAYYQCEES+YMMIIEEaAMMIIAyCMAGEEQOX/NyovEiOMABBG3E/8a5jnJR7I/GsY8e9C5f83Ks/BiCsMgBFgBAgjHsiI5yaMeMGEeU5GPJt4TsKIF0QY8W8hLqPy/xtVGBAgjAAQRoARL5wBEC8qYQCEARDmCiNAGBDPZsS/lTAAYAQI81yo/P+G+A9hxHMy4t/DiCsMiOdmxIvKiPsZ8QBU/n+j8h/EiGcz/3GE+U9D5f83Kv8hxHMSD2SuEP96BsRzMiD+Q1D5/w3x/xuV/9+o/Jcywoj/fAbEv4jK/2+I/yBGGGHEi8KAAAMgwAAIMAACwIAAAyCMACPACAMgwAgwwggAMAKMADAgwAgAIyr/v1F5JiP+7Yy4QhjxghgAYQQYAeJ+AowAAQBGmCsEgDACwAgAAUY8NyMeyAgw4gGo/P9GBQMCDAgjjDDCCCOMMMIII4wwwgCIF84IACNA/McSD2TuZ+5nxHMzwgBU/n+jGnGF+NcSYF4QI4wQRhgBYEA8kAEBYEC8IAYECANGXGFAABhhAIwwAEY8L2GEgcr/b1RhxBVGvDDCiAcyAox4NmGEEUYACCOuEFeY+wkwAgQY8fyJK4wwwggAAUaAuJ+4nzDiBaLy/xsVhBEvCiOMuJ8RAMKI+xkBAsQVRhjxQALMv54RVwgjjHjhhBHPTVxG5f83qgEBwlxhrjACwAgjnk0YADAgXjgjQBgBBsQDGRAABsSzGTBgBBgQz02YF4URzweV/98Q/0ZGPJsRYAQY8R/DCAAj/pNQ+f+Nyn8IYQSAEf9RxBXiPw2V/9+o/BsYEM9JXCH+V6Hy/xvi/zcq/79R+f+Nyv9vVCPACDAgwFwhDIAAMADC3E8YAUYYAWCEEWAEGAFGGGGEEUYYYQQYEFcYcYURRtzPgAAjDIAAMADigYx4kVD5/43KsxgBRoAAAyDACCPAgAAj/uMI8y8xAoy4QoARRoB5ICNeRFT+f6PyryJeEGGEEf82BsTzMiBA/EsMCDDiCgPCAAgAAwLAgIDK/29UMP92wtxPGPFvJcAIMCDuJ8AIMPczzybMFQKMAAPCCAMgrjDCABhhACr/v1FBgHlBDIgXTIABMMKI52b+dQQY8dwEGABhHkgYMPcTBsCIF8RcRuX/Nyr/AvFARjyQEVcYAcKIBxIA5t/LiOfHiPsJ80DCiOdPXEbl/zcqzyIMiOdPGBDPSRgQz80AGPFABswVBgwYAWBAABgQ9zMgQBgQACAMiAcS5gojAMwDGTBghDAgoPL/G+I/kRH/EiP+axgBYMSzUPn/DfH/G5X/36j8/0bl/zcq/79R+f+Nyv9vVP5/o/L/G5X/36j8CwyI/zxG/Nczwgio/P+GjDDCCAMCjAAjjAAjjDACDAgjAIwwwggwIAyAAAPCgAADIIwwwlwhwAgjjDACjDDCCCOMMMIII8CAADACwAgjAAwIMAIMCAMgwAiAyv9vVB5AgBEvjBEGwAjzQEYYAGHACAMgjBBGPJAwwoh/C2FeMAFGABhhAIQBI4yAyv9vVGHACAAjrjD3M1cY8UDCCANg/iXihRNGPCfzLzMgwIC4woAw9zP3E+YKcxmV/9+oRhhhBAgjwAgDYIQBEEY8f8K8cEa8MEYY8WwCzL9EGABh7icMCDAAwgCAEQZAXEbl/zcqD2BAAIj7iRdMPDcjHsgIEOaBhAEQBsxzMmDEsxkwVxgwYASAEQBG3M8IMCDuZ8SzCQMCKv+/If4TGfGCGAFgxPMy4oUz4j8Alf/fqPyPJP4l4j8Elf/fqPw3MCD+R6Dy/xvi/zcq/79R+f+Nyv9vVAAjDAgAIwwIAAMCwIC4nxEARhjx/BnxPxqV/98QGGHEFUYYAQBGXGHE/0FU/n9D5tmEuZ8Ac4UwACDAgDACwAgjwIAwIIwwwoj/0aj8/0YVRhhhQBhhBIAwwoAwwoAR5nkZAUb8r0Ll/zcq/0GEEcKI/0Wo/P9G5V9JGPH8CCOEEf9rUPn/jcq/khHPjwFhQIAw/ytQ+f+NyrOI5088kDAgnpsAEPcT5n8BKv+/UXkRifsJ838Glf/fqLyIDAgwIIwAA8KI52ZAGPE/HJX/3/hHNBI/AbKYmjkAAAAASUVORK5CYII=)
pub const MOGEEFONT: Font<'_> = Font {
    image: ImageRaw::new(include_bytes!("mogeefont.raw"), 128),
    glyph_mapping: StrGlyphMapping::new(
        "\0\u{20}\u{5f}\0\u{61}\u{7e}\u{d7}\u{401}\0\u{410}\u{423}\0\u{425}\u{44f}\u{451}",
        31,
    ),
    glyph_data: include_bytes!("glyph_data.raw"),
    ligature_substitution: StrLigatureSubstitution::new(
        "\0\u{66}\u{66}\u{69}\0\u{66}\u{66}\0\u{66}\u{69}\0\u{66}\u{6a}\0\u{67}\u{6a}\0\u{6a}\u{6a}\0\u{73}\u{73}\0\u{79}\u{6a}",
        160,
    ),
    character_height: 11,
    baseline: 8,
    character_spacing: 1,
};