#![no_std]

#[cfg(test)] extern crate std;

pub const FHEIGHT:usize=11;
pub const FWIDTH:usize=6;
pub const ERRINV:&str="invalid dimensions";
pub const ERRBUFF:&str="buffer too small";
pub const ULINE:u8=1;
pub const STRIKE:u8=2;
pub const FLIPY:u8=4;
const GLYBITS:usize=64;
const fn z(bits:u64,shift:u32)->u64{bits<<shift}

pub const FPIXELS:[u64;224]=[
    0, 0x2082082080082000, z(0x5145, 48), 0x514F94514F945000,
    0x21CAA870AA9C2000, 0x4AA50421052A9000, 0x2145085628A27400, z(0x1041, 49),
    0x0842082082040800, 0x4081041041084000, z(0x54E395, 41), z(0x1047C41, 25),
    z(0x821, 7), z(0x1F, 35), z(0xC3, 12), 0x0821042104208000,
    0x7228A28A28A27000, 0x0862928820820800, 0x722882108420F800, 0x7228823028A27000,
    0x10C51493E1041000, 0xFA083C082082F000, 0x722820F228A27000, 0xF821042084104000,
    0x7228A27228A27000, 0x7228A27820A27000, z(0xC30000C3, 18), 0x00030C00030C1080,
    0x0021084081020000, z(0x1F01F, 29), 0x0204081084200000, 0x7228842080082000,
    0x7A1B75D75D6E81E0, 0x20851453E8A28800, 0xF228A2F228A2F000, 0x7228208208227000,
    0xF228A28A28A2F000, 0xFA0820F20820F800, 0xFA0820F208208000, 0x722820B228A27000,
    0x8A28A2FA28A28800, 0xF88208208208F800, 0xF820820820A27000, 0x8A4928C289248800,
    0x820820820820F800, 0x8B6DAAAA28A28800, 0x8B2CAAAAA9A68800, 0xFA28A28A28A2F800,
    0xF228A2F208208000, 0x7228A28A28A46800, 0xF228A2F289228800, 0x7228207020A27000,
    0xF882082082082000, 0x8A28A28A28A27000, 0x8A28A25145082000, 0x8A28A28AAA945000,
    0x8A25142145228800, 0x8A25142082082000, 0xF82104210420F800, 0x3882082082083800,
    0x8204102041020800, 0xE08208208208E000, z(0x4291, 47), z(0x1F, 11),
    z(0x81, 54), z(0x7027A27, 12), 0x820820F228A2F000, z(0xF41040F, 11),
    0x0820827A28A27800, z(0x722FA07, 12), 0x1882087082082000, 0x0000007A28A27827,
    0x820820F228A28800, 0x0002002082082000, 0x0002002082082284, 0x4104104946144800,
    0x4104104104103000, z(0x1E555555, 11), z(0x1E451451, 11), z(0x7228A27, 12),
    0x0000007124927104, 0x0000007249247041, z(0x1C49041, 14), z(0x3907827, 12),
    0x2082087082081800, z(0x1145144F, 11), z(0x45128A1, 13), z(0x8AA7145, 12),
    z(0x11284291, 11), 0x0000008A25142084, z(0x1F08421F, 11), 0x0841042041040800,
    0x2082082082082000, 0x8104102104108000, z(0x42A1, 30), 0,
    0x7228208208227084, 0x0005008A28A27800, 0x004200722FA07000, 0x0085007027A27000,
    0x0005007027A27000, 0x0102007027A27000, 0x2142007027A27000, 0x0000007A08207884,
    0x008500722FA07000, 0x000500722FA07000, 0x010200722FA07000, 0x0005002082082000,
    0x0085002082082000, 0x0102002082082000, 0xA8851453E8A28800, 0x21421453E8A28800,
    0x108FA083C820F800, z(0x3627FA17, 10), 0x5D4B2CFA49249C00, 0x0085007228A27000,
    0x0005007228A27000, 0x0102007228A27000, 0x0085008A28A27800, 0x0102008A28A27800,
    0x0005008A25142084, 0x500FA28A28A2F800, 0x5008A28A28A27000, 0x002726AB27200000,
    0x390410F10410F800, 0x082726AB27208000, z(0x11284291, 23), 0x188208708208C000,
    0x0042007027A27000, 0x0042002082082000, 0x0042007228A27000, 0x0042008A28A27800,
    0x42A1007124924800, 0x42A122CAAAA68800, 0x604624700F800000, 0x7228A2700F800000,
    0x2080082108A27000, 0x01E86DAEDAE17800, z(0x7C1, 29), 0x4B25142164A4B800,
    0x4B25142125AF8800, 0x2080082082082000, z(0x94A4489, 22), z(0x9122529, 24),
    0x9004802409004800, 0xA95222568095A120, 0xA95A95A95A95A950, 0x2082082082082082,
    0x2082082382082082, 0x108008514FA28800, 0x214008514FA28800, 0x408008514FA28800,
    0x01E867A699E17800, 0x514514D04D145145, 0x5145145145145145, 0x000000F04D145145,
    0x514514D04F000000, 0x08472AA2A7108000, 0x8A251421C21C2000, z(0x1C1041041, 1),
    0x20820820F0000000, 0x20820823F0000000, z(0x1F9041041, 1), 0x20820820F2082082,
    z(0x3F, 28), 0x20820823F2082082, 0x42A1007027A27000, 0x42A108514FA28800,
    0x5145145D07C00000, 0x0000007D05D45145, 0x514514DC0FC00000, 0x000000FC0DD45145,
    0x5145145D05D45145, z(0x3F03F, 22), 0x514514DC0DD45145, 0x0227228A27220000,
    0xA10A047228A27000, 0x712492E924927000, 0x214FA083C820F800, 0x500FA083C820F800,
    0x408FA083C820F800, z(0xC10411F, 11), 0x10803E208208F800, 0x21403E208208F800,
    0x01403E208208F800, z(0x41041047, 31), z(0x79041041, 1), 0xFFFFFFFFFFFFFFF0,
    z(0x3FFFFFFF, 4), 0x2082080002082080, 0x40803E208208F800, z(0x3FFFFFFF, 34),
    0x10803E8A28A2F800, 0x3124944925104000, 0x21403E8A28A2F800, 0x40803E8A28A2F800,
    0x42A1007228A27000, 0x42A13E8A28A2F800, 0x0000008A28A2FA08, 0x82082CCA28A2F208,
    0x000820F228A2F208, 0x1088A28A28A2F800, 0x2140228A28A2F800, 0x4088A28A28A2F800,
    0x0042008A25142084, 0x12A8945082082000, z(0x1F, 59), z(0x21, 55),
    z(0x7, 36), 0x00823E20803E0000, z(0x1F01F, 5), 0xC49C8AD0A3974800,
    0x7BAEBA6820A27000, 0x31240C49230248C0, z(0x1007C01, 25), z(0x41, 7),
    z(0x7147, 48), z(0x5, 54), z(0x1, 37), z(0x2182087, 36),
    z(0x3023023, 37), z(0x6042107, 36), z(0x3FFFFFFF, 22), 0,
];
// Atlas slots 0x80..0xFF follow CP850, not Unicode/Latin-1.
// https://www.unicode.org/Public/MAPPINGS/VENDORS/MICSFT/PC/CP850.TXT
const EXTENDED_CHARS: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç', 'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù', 'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '×', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º', '¿', '®', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À', '©', '╣', '║', '╗', '╝', '¢', '¥', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã', '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'ð', 'Ð', 'Ê', 'Ë', 'È', 'ı', 'Í', 'Î', 'Ï', '┘', '┌', '█', '▄', '¦', 'Ì', '▀',
    'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ', 'Þ', 'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´',
    '\u{ad}', '±', '‗', '¾', '¶', '§', '÷', '¸', '°', '¨', '·', '¹', '³', '²', '■', '\u{a0}',
];

/// Return the existing atlas slot for one Unicode character. Unsupported
/// characters occupy one replacement cell; byte APIs retain raw atlas indexing.
pub fn glyph_byte(character: char) -> u8 {
    if character.is_ascii() {
        character as u8
    } else {
        EXTENDED_CHARS.iter().position(|&entry| entry == character)
            .map_or(b'?', |index| 0x80 + index as u8)
    }
}
pub fn font_pixels(character:u8)->u64{
    FPIXELS[(i32::from(character)-32).clamp(0,FPIXELS.len() as i32-1) as usize]
}
pub fn measure_text(text:&str)->(usize,usize){(text.chars().count()*FWIDTH,FHEIGHT)}
pub fn measure_bytes(bytes:&[u8])->(usize,usize){(bytes.len()*FWIDTH,FHEIGHT)}
pub fn stamp_text<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    x: i32,
    y: i32,
    text: &str,
    color: T,
) -> Result<(), &'static str> {
    stamp_text_with_stride(buffer, width, height, width, x, y, text, color)
}
pub fn stamp_bytes<T: Copy>(
    buffer: &mut [T], width: usize, height: usize, x: i32, y: i32, bytes: &[u8], color: T
) -> Result<(), &'static str> {
    stamp_bytes_with_stride(buffer,width,height,width,x,y,bytes,color)
}
pub fn stamp_text_with_stride<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    text: &str,
    color: T,
) -> Result<(), &'static str> {
    stamp_glyphs_with_stride(buffer, width, height, stride, x, y, text.chars().map(glyph_byte), color)
}
pub fn stamp_bytes_with_stride<T: Copy>(
    buffer: &mut [T],
    width: usize,
    height: usize,
    stride: usize,
    x: i32,
    y: i32,
    bytes: &[u8],
    color: T,
) -> Result<(), &'static str> {
    stamp_glyphs_with_stride(buffer, width, height, stride, x, y, bytes.iter().copied(), color)
}
fn stamp_glyphs_with_stride<T: Copy>(
    buffer: &mut [T], width: usize, height: usize, stride: usize,
    x: i32, y: i32, glyphs: impl Iterator<Item = u8>, color: T,
) -> Result<(), &'static str> {
    if stride < width {
        return Err(ERRINV);
    }

    let required_len = if width == 0 || height == 0 {
        0
    } else {
        (height - 1)
            .checked_mul(stride)
            .and_then(|row| row.checked_add(width))
            .ok_or(ERRINV)?
    };
    if buffer.len() < required_len {
        return Err(ERRBUFF);
    }

    let width_i32 = i32::try_from(width).map_err(|_| ERRINV)?;
    let height_i32 = i32::try_from(height).map_err(|_| ERRINV)?;

    for (index, character) in glyphs.enumerate() {
        let pixels = font_pixels(character);
        let x_bias = i32::from(character == b'q');
        let glyph_x = x + index as i32 * FWIDTH as i32 + x_bias;

        for bit in 0..GLYBITS {
            if (pixels >> (63 - bit)) & 1 == 0 {
                continue;
            }

            let px = glyph_x + (bit % FWIDTH) as i32;
            let py = y + (bit / FWIDTH) as i32;
            if px >= 0 && py >= 0 && px < width_i32 && py < height_i32 {
                buffer[py as usize * stride + px as usize] = color;
            }
        }
    }
    Ok(())
}
pub fn stamp_argb(
    buffer:&mut [u32],width:usize,height:usize,x:i32,y:i32,bytes:&[u8],argb:u32,style:u8
)->Result<(),&'static str>{
    stamp_argb_stride(buffer,width,height,width,x,y,bytes,argb,style)
}
pub fn stamp_argb_stride(
    buffer:&mut [u32],width:usize,height:usize,stride:usize,x:i32,y:i32,bytes:&[u8],argb:u32,style:u8
)->Result<(),&'static str>{
    if stride<width{return Err(ERRINV);}
    let required_len=if width==0||height==0{0}else{(height-1).checked_mul(stride).and_then(|row|row.checked_add(width)).ok_or(ERRINV)?};
    if buffer.len()<required_len{return Err(ERRBUFF);}
    let width_i32=i32::try_from(width).map_err(|_|ERRINV)?;
    let height_i32=i32::try_from(height).map_err(|_|ERRINV)?;
    for (index,character) in bytes.iter().copied().enumerate(){
        let pixels=font_pixels(character);
        let x_bias=i32::from(character==b'q');
        let glyph_x=x+index as i32*FWIDTH as i32+x_bias;
        for bit in 0..GLYBITS{
            if (pixels>>(63-bit))&1==0{continue;}
            let row=(bit/FWIDTH) as i32;
            let py=y+if style&FLIPY!=0{FHEIGHT as i32-1-row}else{row};
            let px=glyph_x+(bit%FWIDTH) as i32;
            put_argb(buffer,width_i32,height_i32,stride,px,py,argb);
        }
    }
    let w=bytes.len() as i32*FWIDTH as i32;
    if style&STRIKE!=0{hline_argb(buffer,width_i32,height_i32,stride,x,y+FHEIGHT as i32/2,w,argb);}
    if style&ULINE!=0{hline_argb(buffer,width_i32,height_i32,stride,x,y+FHEIGHT as i32,w,argb);}
    Ok(())
}
fn put_argb(buffer:&mut [u32],width:i32,height:i32,stride:usize,x:i32,y:i32,argb:u32){
    if x>=0&&y>=0&&x<width&&y<height{buffer[y as usize*stride+x as usize]=argb;}
}
fn hline_argb(buffer:&mut [u32],width:i32,height:i32,stride:usize,x:i32,y:i32,w:i32,argb:u32){
    for dx in 0..w{put_argb(buffer,width,height,stride,x+dx,y,argb);}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File,create_dir_all};
    use std::io::Write;
    use std::vec;

    #[test]
    fn exposes_upstream_font_pixels() {
        assert_eq!(FWIDTH, 6);
        assert_eq!(FHEIGHT, 11);
        assert_eq!(font_pixels(b'"'), 0x5145000000000000);
        assert_eq!(font_pixels(b'A'), 0x20851453E8A28800);
        assert_eq!(font_pixels(b'0'), 0x7228A28A28A27000);
        assert_eq!(font_pixels(0), font_pixels(b' '));
        assert_eq!(font_pixels(255), 0);
    }
    #[test]
    fn unicode_uses_existing_cp850_slots_and_one_cell_per_character() {
        assert_eq!(glyph_byte('§'), 0xF5);
        assert_eq!(font_pixels(glyph_byte('§')), 0x31240C49230248C0);
        assert_eq!(glyph_byte('ä'), 0x84);
        assert_eq!(glyph_byte('é'), 0x82);
        assert_eq!(glyph_byte('─'), 0xC4);
        assert_eq!(glyph_byte('🙂'), b'?');
        for (index, &character) in EXTENDED_CHARS.iter().enumerate() {
            assert_eq!(glyph_byte(character), 0x80 + index as u8);
        }
        let text = "§äé─🙂A";
        let (width, height) = measure_text(text);
        assert_eq!((width, height), (6 * FWIDTH, FHEIGHT));
        let mut unicode = vec![0u8; width * height];
        let mut bytes = unicode.clone();
        stamp_text(&mut unicode, width, height, 0, 0, text, 1).unwrap();
        stamp_bytes(&mut bytes, width, height, 0, 0, &[0xF5, 0x84, 0x82, 0xC4, b'?', b'A'], 1).unwrap();
        assert_eq!(unicode, bytes);
    }
    #[test]
    fn unicode_stride_and_clipping_match_raw_atlas_glyphs() {
        let mut unicode = vec![9u8; 20 * FHEIGHT];
        let mut bytes = unicode.clone();
        stamp_text_with_stride(&mut unicode, 12, FHEIGHT, 20, -2, -1, "§é", 1).unwrap();
        stamp_bytes_with_stride(&mut bytes, 12, FHEIGHT, 20, -2, -1, &[0xF5, 0x82], 1).unwrap();
        assert_eq!(unicode, bytes);
        assert!(unicode.chunks_exact(20).all(|row| row[12..].iter().all(|&p| p == 9)));
    }
    #[test]
    fn stamps_text_into_a_flat_buffer() {
        let mut buffer = vec![0u32; 12 * 12];

        stamp_text(&mut buffer, 12, 12, 0, 0, "A", 0x00FF00).unwrap();

        assert_eq!(
            buffer.iter().filter(|pixel| **pixel == 0x00FF00).count(),
            font_pixels(b'A').count_ones() as usize
        );
    }
    #[test]
    fn clips_text_at_buffer_edges() {
        let mut buffer = vec![0u8; 8 * 8];

        stamp_text(&mut buffer, 8, 8, -2, -1, "A", 1).unwrap();

        let lit_pixels = buffer.iter().filter(|pixel| **pixel == 1).count();
        assert!(lit_pixels > 0);
        assert!(lit_pixels < font_pixels(b'A').count_ones() as usize);
    }
    #[test]
    fn rejects_short_buffers() {
        let mut buffer = vec![0u8; 3];

        let err = stamp_text(&mut buffer, 2, 2, 0, 0, "A", 1).unwrap_err();

        assert_eq!(err, ERRBUFF);
    }

    #[test]
    fn style_alpha_and_flip_are_stamped() {
        let mut line=vec![0xFF101820u32; 16*16];
        stamp_argb(&mut line,16,16,2,2,b" ",0x80FF0000,ULINE|STRIKE).unwrap();
        assert_eq!(line[(2+FHEIGHT/2)*16+2]>>24,0x80);
        assert_eq!(line[(2+FHEIGHT)*16+2],0x80FF0000);

        let mut a=vec![0u32; FWIDTH*FHEIGHT];
        let mut b=vec![0u32; FWIDTH*FHEIGHT];
        stamp_argb(&mut a,FWIDTH,FHEIGHT,0,0,b"A",0x8000FF00,0).unwrap();
        stamp_argb(&mut b,FWIDTH,FHEIGHT,0,0,b"A",0x8000FF00,FLIPY).unwrap();
        for y in 0..FHEIGHT{for x in 0..FWIDTH{
            assert_eq!(a[y*FWIDTH+x],b[(FHEIGHT-1-y)*FWIDTH+x]);
        }}
        assert!(a.iter().any(|p|*p==0x8000FF00));
    }

    #[test]
    fn writes_separate_font_atlas_bmps() {
        let plain=atlas(0,0xFFFF3040);
        let alpha=atlas(0,0x80FF3040);
        let under=atlas(ULINE,0xFFFF3040);
        let strike=atlas(STRIKE,0xFFFF3040);
        let flip=atlas(FLIPY,0xFFFF3040);
        let (w,h)=atlas_dims();

        assert!(plain.iter().any(|p|*p==0xFFFF3040));
        assert!(alpha.iter().any(|p|(*p>>24)==0x80));
        assert!(under.iter().filter(|p|**p!=0xFF101820).count()>plain.iter().filter(|p|**p!=0xFF101820).count());
        assert!(strike.iter().filter(|p|**p!=0xFF101820).count()>plain.iter().filter(|p|**p!=0xFF101820).count());
        assert_ne!(plain,flip);

        create_dir_all("target").unwrap();
        write_bmp24_argb("target/font-atlas.bmp",w,h,&plain).unwrap();
        write_bmp24_argb("target/font-atlas-alpha.bmp",w,h,&alpha).unwrap();
        write_bmp24_argb("target/font-atlas-underline.bmp",w,h,&under).unwrap();
        write_bmp24_argb("target/font-atlas-strike.bmp",w,h,&strike).unwrap();
        write_bmp24_argb("target/font-atlas-flip.bmp",w,h,&flip).unwrap();
        let doubled=scale2(&plain,w,h);
        write_bmp24_argb("target/font-atlas-2x.bmp",w*2,h*2,&doubled).unwrap();
        assert_eq!(doubled.len(),plain.len()*4);
    }

    fn atlas(style:u8,base:u32)->std::vec::Vec<u32>{
        let (w,h)=atlas_dims();
        let cols=16usize; let cell_w=FWIDTH+4; let cell_h=FHEIGHT+3;
        let mut img=vec![0xFF101820u32;w*h];
        let rgb=[0xFF3040,0xFF9D2E,0xFFE83A,0x2FE66B,0x2BC7FF,0x635BFF,0xD93DFF];
        for i in 0..FPIXELS.len(){
            let x=(i%cols)*cell_w+1;
            let y=(i/cols)*cell_h+1;
            let color=(base&0xFF000000)|rgb[(i/10)%rgb.len()];
            stamp_argb(&mut img,w,h,x as i32,y as i32,&[(i+32) as u8],color,style).unwrap();
        }
        img
    }
    fn atlas_dims()->(usize,usize){
        let cols=16usize; let rows=FPIXELS.len().div_ceil(cols);
        (cols*(FWIDTH+4),rows*(FHEIGHT+3))
    }
    fn write_bmp24_argb(path:&str,w:usize,h:usize,img:&[u32])->std::io::Result<()>{
        let row=(w*3).div_ceil(4)*4;
        let bytes=row*h;
        let mut f=File::create(path)?;
        f.write_all(b"BM")?;
        wu32(&mut f,(54+bytes) as u32)?;
        wu16(&mut f,0)?; wu16(&mut f,0)?; wu32(&mut f,54)?;
        wu32(&mut f,40)?; wi32(&mut f,w as i32)?; wi32(&mut f,h as i32)?;
        wu16(&mut f,1)?; wu16(&mut f,24)?; wu32(&mut f,0)?; wu32(&mut f,bytes as u32)?;
        wi32(&mut f,2835)?; wi32(&mut f,2835)?; wu32(&mut f,0)?; wu32(&mut f,0)?;
        let pad=vec![0u8;row-w*3];
        for y in (0..h).rev(){
            for x in 0..w{let [r,g,b]=argb_to_rgb(img[y*w+x]); f.write_all(&[b,g,r])?;}
            f.write_all(&pad)?;
        }
        Ok(())
    }
    fn argb_to_rgb(p:u32)->[u8;3]{
        let a=p>>24; let bg=[0x10u32,0x18,0x20];
        let rgb=[(p>>16)&255,(p>>8)&255,p&255];
        [
            ((rgb[0]*a+bg[0]*(255-a)+127)/255) as u8,
            ((rgb[1]*a+bg[1]*(255-a)+127)/255) as u8,
            ((rgb[2]*a+bg[2]*(255-a)+127)/255) as u8,
        ]
    }
    fn scale2(img:&[u32],w:usize,h:usize)->std::vec::Vec<u32>{
        let mut out=vec![0u32;w*h*4];
        for y in 0..h{for x in 0..w{
            let p=img[y*w+x]; let ox=x*2; let oy=y*2; let ow=w*2;
            out[oy*ow+ox]=p; out[oy*ow+ox+1]=p; out[(oy+1)*ow+ox]=p; out[(oy+1)*ow+ox+1]=p;
        }}
        out
    }
    fn wu16(f:&mut File,v:u16)->std::io::Result<()>{f.write_all(&v.to_le_bytes())}
    fn wu32(f:&mut File,v:u32)->std::io::Result<()>{f.write_all(&v.to_le_bytes())}
    fn wi32(f:&mut File,v:i32)->std::io::Result<()>{f.write_all(&v.to_le_bytes())}
}
