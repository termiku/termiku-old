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
    CursorUp(usize),
    
    /// CUD  
    /// CSI Pn 04/02  
    /// CSI Pn 0x41  
    /// CSI Pn B  
    ///   
    /// Pn = 0x01  
    CursorDown(usize),
    
    /// CUF  
    /// CSI Pn 04/03  
    /// CSI Pn 0x43  
    /// CSI Pn C  
    ///   
    /// Pn = 0x01  
    CursorRight(usize),
    
    /// CUB  
    /// CSI Pn 04/04  
    /// CSI Pn 0x44  
    /// CSI Pn D  
    ///   
    /// Pn = 0x01  
    CursorLeft(usize),
    
    /// CNL  
    /// CSI Pn 04/05  
    /// CSI Pn 0x45  
    /// CSI Pn E  
    ///   
    /// Pn = 0x01  
    CursorNextLine(usize),
    
    /// CPL  
    /// CSI Pn 04/06  
    /// CSI Pn 0x46  
    /// CSI Pn F  
    ///   
    /// Pn = 0x01  
    CursorPrecedingLine(usize),
    
    /// CHA  
    /// CSI Pn 04/07  
    /// CSI Pn 0x47  
    /// CSI Pn G  
    ///   
    /// Pn = 0x01  
    CursorCharacterAbsolute(usize),
    
    
    /// CUP  
    /// CSI Pn1;Pn2 04/08  
    /// CSI Pn1;Pn2 0x048  
    /// CSI Pn1;Pn2 H  
    ///   
    /// Pn1 = 0x01  
    /// Pn2 = 0x01  
    CursorPosition(usize, usize)
}