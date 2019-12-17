/// Define all recognized control sequences, unless precised, as defined by ECMA-48  
///   
/// Syntax:  
///   
/// ECMA-48 Acronym  
/// ECMA-48 Representation (searchable in the standard)  
/// ECMA-48 Representation, in hexadecimal  
/// ECMA-48 Representation, interpreted as characters  
///   
/// Default parameters values, in hexadecimal  
#[derive(Debug)]
pub enum ControlType {
    Unknown,
    
    /// CUU  
    /// CSI Pn 04/01  
    /// CSI Pn 0x41  
    /// CSI Pn A  
    ///   
    /// Pn = 0x01
    CursorUp(u16),
    
    /// CUD  
    /// CSI Pn 04/02  
    /// CSI Pn 0x41  
    /// CSI Pn B  
    ///   
    /// Pn = 0x01  
    CursorDown(u16),
    
    /// CUF  
    /// CSI Pn 04/03  
    /// CSI Pn 0x43  
    /// CSI Pn C  
    ///   
    /// Pn = 0x01  
    CursorRight(u16),
    
    /// CUB  
    /// CSI Pn 04/04  
    /// CSI Pn 0x44  
    /// CSI Pn D  
    ///   
    /// Pn = 0x01  
    CursorLeft(u16),
    
    /// CNL  
    /// CSI Pn 04/05  
    /// CSI Pn 0x45  
    /// CSI Pn E  
    ///   
    /// Pn = 0x01  
    CursorNextLine(u16),
    
    /// CPL  
    /// CSI Pn 04/06  
    /// CSI Pn 0x46  
    /// CSI Pn F  
    ///   
    /// Pn = 0x01  
    CursorPrecedingLine(u16),
    
    /// CHA  
    /// CSI Pn 04/07  
    /// CSI Pn 0x47  
    /// CSI Pn G  
    ///   
    /// Pn = 0x01  
    CursorCharacterAbsolute(u16),
    
    
    /// CUP  
    /// CSI Pn1;Pn2 04/08  
    /// CSI Pn1;Pn2 0x48  
    /// CSI Pn1;Pn2 H  
    ///   
    /// Pn1 = 0x01  
    /// Pn2 = 0x01  
    CursorPosition(u16, u16),
    
    /// ED  
    /// CSI Ps 04/10  
    /// CSI Ps 0x41  
    /// CSI Ps J
    /// 
    /// Pn = 0x00
    EraseInPage(u16),
    
    /// DL  
    /// CSI Pn 04/13  
    /// CSI Pn 0x4D  
    /// CSI Pn M  
    ///   
    /// Pn = 0x01  
    DeleteLine(u16),
    
    /// SGR
    /// CSI Ps... 06/13
    /// CSI Ps... 0x6D
    /// CSI Ps... m
    /// 
    /// Ps = 0
    SelectGraphicRendition(Vec<u16>),
    
    /// ???  
    /// CSI 0x73  
    /// CSI s  
    ///   
    /// Note: Not documented by ECMA-48  
    SaveCursor,
    
    /// ???  
    /// CSI 0x75  
    /// CSI u  
    ///   
    /// Note: Not documented by ECMA-48  
    RestoreCursor
}