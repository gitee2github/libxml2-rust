use super::parserInternals::*;
use rust_ffi::ffi_defination::defination::*;
use rust_ffi::ffi_extern_method::extern_method::*;
use rust_ffi::ffi_extern_method::extern_method_safe::*;
use std::mem::size_of;

fn IS_BLANK_CH(cur: *const xmlChar) -> bool {
    return unsafe { *cur } as i32 == 0x20 as i32
        || 0x9 as i32 <= unsafe { *cur } as i32 && unsafe { *cur } as i32 <= 0xa as i32
        || unsafe { *cur } as i32 == 0xd as i32;
}
const INPUT_CHUNK: i32 = 250;
const XML_MAX_LOOKUP_LIMIT: i64 = 10000000;
const XML_PARSER_BIG_BUFFER_SIZE: u64 = 300;
fn IS_CHAR(q: i32) -> bool {
    if q < 0x100 {
        (0x9 <= q && q <= 0xa) || q == 0xd || 0x20 <= q
    } else {
        (0x100 <= q && q <= 0xd7ff)
            || (0xe000 <= q && q <= 0xfffd)
            || (0x10000 <= q && q <= 0x10ffff)
    }
}

fn GROW(ctxt: xmlParserCtxtPtr) {
    if unsafe { (*ctxt).progressive } == 0
        && unsafe { ((*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
}

fn SKIP(ctxt: xmlParserCtxtPtr, val: i32) {
    unsafe {
        (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(val as isize);
        (*(*ctxt).input).col += val;
    }
    if unsafe { *(*(*ctxt).input).cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe((*ctxt).input, INPUT_CHUNK) };
    }
}

fn IS_LETTER(c: i32, group: *const xmlChRangeGroup) -> bool {
    ((if c < 0x100 {
        (0x41 <= c && c <= 0x5a
            || 0x61 <= c && c <= 0x7a
            || 0xc0 <= c && c <= 0xd6
            || 0xd8 <= c && c <= 0xf6
            || 0xf8 <= c) as i32
    } else {
        unsafe { xmlCharInRange_safe(c as u32, group) }
    }) != 0
        || (if c < 0x100 {
            0
        } else {
            (0x4e00 <= c && c <= 0x9fa5 || c == 0x3007 || 0x3021 <= c && c <= 0x3029) as i32
        }) != 0)
}

fn IS_DIGIT(c: i32, group: *const xmlChRangeGroup) -> bool {
    (if c < 0x100 {
        (0x30 <= c && c <= 0x39) as i32
    } else {
        unsafe { xmlCharInRange_safe(c as u32, group) }
    }) != 0
}

fn IS_COMBINING(c: i32, group: *const xmlChRangeGroup) -> bool {
    (if c < 0x100 {
        0
    } else {
        unsafe { xmlCharInRange_safe(c as u32, group) }
    }) != 0
}

fn IS_EXTENDER(c: i32, group: *const xmlChRangeGroup) -> bool {
    (if c < 0x100 {
        (c == 0xb7) as i32
    } else {
        unsafe { xmlCharInRange_safe(c as u32, group) }
    }) != 0
}

fn NEXTL(ctxt: htmlParserCtxtPtr, ql: i32) {
    let ctxtPtr = unsafe { &mut *ctxt };
    let inputPtr = unsafe { &mut *(*ctxt).input };
    if CUR(ctxt) == '\n' as i32 {
        inputPtr.line += 1;
        inputPtr.col = 1
    } else {
        inputPtr.col += 1
    }
    ctxtPtr.token = 0;
    unsafe {
        inputPtr.cur = inputPtr.cur.offset(ql as isize);
    }
}

fn CUR(ctxt: htmlParserCtxtPtr) -> i32 {
    unsafe { *(*(*ctxt).input).cur as i32 }
}

fn COPY_BUF(ql: i32, buf: *mut xmlChar, mut len: i32, q: i32) -> i32 {
    if ql == 1 {
        let fresh40 = len;
        len = len + 1;
        unsafe {
            *buf.offset(fresh40 as isize) = q as xmlChar;
        }
    } else {
        unsafe {
            len += xmlCopyChar_safe(ql, buf.offset(len as isize), q);
        }
    }
    return len;
}

fn SHRINK(ctxt: htmlParserCtxtPtr) {
    let ctxtPtr = unsafe { &mut *ctxt };
    if ctxtPtr.progressive == 0
        && SHRINK_bool1(ctxt, (2 * INPUT_CHUNK) as i64)
        && SHRINK_bool2(ctxt, (2 * INPUT_CHUNK) as i64)
    {
        unsafe { xmlParserInputShrink_safe(ctxtPtr.input) };
    }
}

fn SHRINK_bool1(ctxt: htmlParserCtxtPtr, num: i64) -> bool {
    let result: i64 = unsafe { (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) } as i64;
    result > num
}

fn SHRINK_bool2(ctxt: htmlParserCtxtPtr, num: i64) -> bool {
    let mut result: i64 = unsafe { (*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) } as i64;
    result < num
}

/* *
* nameNsPush:
* @ctxt:  an XML parser context
* @value:  the element name
* @prefix:  the element prefix
* @URI:  the element namespace name
* @line:  the current line number for error messages
* @nsNr:  the number of namespaces pushed on the namespace table
*
* Pushes a new element name/prefix/URL on top of the name stack
*
* Returns -1 in case of error, the index in the stack otherwise
*/
fn nameNsPush(
    ctxt: xmlParserCtxtPtr,
    value: *const xmlChar,
    prefix: *const xmlChar,
    URI: *const xmlChar,
    line: i32,
    nsNr: i32,
) -> i32 {
    let current_block: u64;
    let mut tag: *mut xmlStartTag;
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).nameNr >= (safe_ctxt).nameMax {
        let tmp: *mut *const xmlChar;
        let tmp2: *mut xmlStartTag;
        (safe_ctxt).nameMax *= 2;
        tmp = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).nameTab as *mut (),
                ((safe_ctxt).nameMax as u64) * (size_of::<*const xmlChar>() as u64),
            )
        } as *mut *const xmlChar;
        if tmp.is_null() {
            (safe_ctxt).nameMax /= 2;
            current_block = 1;
        } else {
            (safe_ctxt).nameTab = tmp;
            tmp2 = unsafe {
                xmlRealloc_safe(
                    (safe_ctxt).pushTab as *mut (),
                    ((safe_ctxt).nameMax as u64) * (size_of::<xmlStartTag>() as u64),
                )
            } as *mut xmlStartTag;
            if tmp2.is_null() {
                (safe_ctxt).nameMax /= 2;
                current_block = 1;
            } else {
                (safe_ctxt).pushTab = tmp2;
                current_block = 2;
            }
        }
    } else if (safe_ctxt).pushTab.is_null() {
        (safe_ctxt).pushTab =
            unsafe { xmlMalloc_safe((safe_ctxt).nameMax as u64 * size_of::<xmlStartTag>() as u64) }
                as *mut xmlStartTag;
        if (safe_ctxt).pushTab.is_null() {
            current_block = 1;
        } else {
            current_block = 2;
        }
    } else {
        current_block = 2;
    }
    match current_block {
        1 => {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return -1;
        }
        _ => {
            unsafe {
                *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize) = value;
                (safe_ctxt).name = value;
                tag = &mut *(*ctxt).pushTab.offset((safe_ctxt).nameNr as isize) as *mut xmlStartTag;
                (*tag).prefix = prefix;
                (*tag).URI = URI;
                (*tag).line = line;
                (*tag).nsNr = nsNr;
            }
            let res = (safe_ctxt).nameNr;
            (safe_ctxt).nameNr = (safe_ctxt).nameNr + 1;
            return res;
        }
    };
}
/* *
* nameNsPop:
* @ctxt: an XML parser context
*
* Pops the top element/prefix/URI name from the name stack
*
* Returns the name just removed
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
fn nameNsPop(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let ret: *const xmlChar;
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).nameNr <= 0 {
        return 0 as *const xmlChar;
    }
    (safe_ctxt).nameNr -= 1;
    if (safe_ctxt).nameNr > 0 {
        (safe_ctxt).name = unsafe { *(*ctxt).nameTab.offset(((safe_ctxt).nameNr - 1) as isize) };
    } else {
        (safe_ctxt).name = 0 as *const xmlChar
    }
    ret = unsafe { *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize) };
    unsafe {
        *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize) = 0 as *const xmlChar;
    }
    return ret;
}
/* LIBXML_PUSH_ENABLED */
/* *
* namePush:
* @ctxt:  an XML parser context
* @value:  the element name
*
* Pushes a new element name on top of the name stack
*
* Returns -1 in case of error, the index in the stack otherwise
*/

pub fn namePush(ctxt: xmlParserCtxtPtr, value: *const xmlChar) -> i32 {
    if ctxt.is_null() {
        return -(1);
    }
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).nameNr >= (safe_ctxt).nameMax {
        let tmp: *mut *const xmlChar;
        tmp = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).nameTab as *mut (),
                ((safe_ctxt).nameMax * 2) as u64 * size_of::<*const xmlChar>() as u64,
            )
        } as *mut *const xmlChar;
        if tmp.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return -(1);
        } else {
            (safe_ctxt).nameTab = tmp;
            (safe_ctxt).nameMax *= 2
        }
    }
    unsafe {
        *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize) = value;
    }
    (safe_ctxt).name = value;
    let res = (safe_ctxt).nameNr;
    (safe_ctxt).nameNr = (safe_ctxt).nameNr + 1;
    return res;
}
/* *
* namePop:
* @ctxt: an XML parser context
*
* Pops the top element name from the name stack
*
* Returns the name just removed
*/

pub fn namePop(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let ret: *const xmlChar;
    if ctxt.is_null() || unsafe { (*ctxt).nameNr <= 0 } {
        return 0 as *const xmlChar;
    }
    let safe_ctxt = unsafe { &mut *ctxt };
    (safe_ctxt).nameNr -= 1;
    if (safe_ctxt).nameNr > 0 {
        (safe_ctxt).name = unsafe { *(*ctxt).nameTab.offset(((safe_ctxt).nameNr - 1) as isize) };
    } else {
        (safe_ctxt).name = 0 as *const xmlChar
    }
    unsafe {
        ret = *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize);
        *(*ctxt).nameTab.offset((safe_ctxt).nameNr as isize) = 0 as *const xmlChar;
    }
    return ret;
}
fn spacePush(ctxt: xmlParserCtxtPtr, val: i32) -> i32 {
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).spaceNr >= (safe_ctxt).spaceMax {
        let tmp: *mut i32;
        (safe_ctxt).spaceMax *= 2;
        tmp = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).spaceTab as *mut (),
                (safe_ctxt).spaceMax as u64 * size_of::<i32>() as u64,
            )
        } as *mut i32;
        if tmp.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            (safe_ctxt).spaceMax /= 2;
            return -(1);
        }
        (safe_ctxt).spaceTab = tmp
    }
    unsafe {
        *(*ctxt).spaceTab.offset((safe_ctxt).spaceNr as isize) = val;
        (safe_ctxt).space = &mut *(*ctxt).spaceTab.offset((safe_ctxt).spaceNr as isize) as *mut i32;
    }
    let res = (safe_ctxt).spaceNr;
    (safe_ctxt).spaceNr = (safe_ctxt).spaceNr + 1;
    return res;
}
fn spacePop(ctxt: xmlParserCtxtPtr) -> i32 {
    let ret: i32;
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).spaceNr <= 0 {
        return 0;
    }
    (safe_ctxt).spaceNr -= 1;
    if (safe_ctxt).spaceNr > 0 {
        (safe_ctxt).space = unsafe {
            &mut *(*ctxt).spaceTab.offset(((safe_ctxt).spaceNr - 1) as isize) as *mut i32
        };
    } else {
        (safe_ctxt).space = unsafe { &mut *(*ctxt).spaceTab.offset(0) as *mut i32 };
    }
    unsafe {
        ret = *(*ctxt).spaceTab.offset((safe_ctxt).spaceNr as isize);
        *(*ctxt).spaceTab.offset((safe_ctxt).spaceNr as isize) = -(1);
    }
    return ret;
}
fn xmlSHRINK(ctxt: xmlParserCtxtPtr) {
    let safe_ctxt = unsafe { &mut *ctxt };
    unsafe { xmlParserInputShrink_safe((safe_ctxt).input) };
    if unsafe { *(*(*ctxt).input).cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe((safe_ctxt).input, 0) };
    };
}
fn xmlGROW(ctxt: xmlParserCtxtPtr) {
    let safe_ctxt = unsafe { &mut *ctxt };
    let curEnd: ptrdiff_t =
        unsafe { (*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64 };
    let curBase: ptrdiff_t =
        unsafe { (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64 };
    if (curEnd > XML_MAX_LOOKUP_LIMIT || curBase > XML_MAX_LOOKUP_LIMIT)
        && unsafe {
            (!(*(*ctxt).input).buf.is_null()
                && (*(*(*ctxt).input).buf).readcallback != Some(xmlInputReadCallbackNop))
        }
        && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
    {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"Huge input lookup\x00" as *const u8 as *const i8,
            );
            xmlHaltParser(ctxt);
        }
        return;
    }
    unsafe { xmlParserInputGrow_safe((safe_ctxt).input, INPUT_CHUNK) };
    if unsafe {
        (*(*ctxt).input).cur > (*(*ctxt).input).end || (*(*ctxt).input).cur < (*(*ctxt).input).base
    } {
        unsafe {
            xmlHaltParser(ctxt);
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"cur index out of bound\x00" as *const u8 as *const i8,
            );
        }
        return;
    }
    if unsafe { !(*(*ctxt).input).cur.is_null() && *(*(*ctxt).input).cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe((safe_ctxt).input, INPUT_CHUNK) };
    };
}
/* *
* xmlSkipBlankChars:
* @ctxt:  the XML parser context
*
* skip all blanks character found at that point in the input streams.
* It pops up finished entities in the process if allowable at that point.
*
* Returns the number of space chars skipped
*/

pub fn xmlSkipBlankChars(ctxt: xmlParserCtxtPtr) -> i32 {
    let mut res: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    /*
     * It's Okay to use CUR/NEXT here since all the blanks are on
     * the ASCII range.
     */
    if (safe_ctxt).instate as i32 != XML_PARSER_DTD as i32 {
        let mut cur: *const xmlChar;
        /*
         * if we are in the document content, go really fast
         */
        cur = unsafe { (*(*ctxt).input).cur };
        while IS_BLANK_CH(cur) {
            if unsafe { *cur } == '\n' as u8 {
                unsafe {
                    (*(*ctxt).input).line += 1;
                    (*(*ctxt).input).col = 1;
                }
            } else {
                unsafe { (*(*ctxt).input).col += 1 }
            }
            cur = unsafe { cur.offset(1) };
            res += 1;
            if unsafe { *cur } as i32 == 0 {
                unsafe {
                    (*(*ctxt).input).cur = cur;
                    xmlParserInputGrow_safe((safe_ctxt).input, INPUT_CHUNK);
                    cur = (*(*ctxt).input).cur;
                }
            }
        }
        unsafe {
            (*(*ctxt).input).cur = cur;
        }
    } else {
        let expandPE: i32 = ((safe_ctxt).external != 0 || (safe_ctxt).inputNr != 1) as i32;
        loop {
            if IS_BLANK_CH(unsafe { (*(*ctxt).input).cur }) {
                /* CHECKED tstblanks.xml */
                unsafe { xmlNextChar_safe(ctxt) };
            } else if unsafe { *(*(*ctxt).input).cur == '%' as u8 } {
                /*
                 * Need to handle support of entities branching here
                 */
                if expandPE == 0
                    || IS_BLANK_CH(unsafe { (*(*ctxt).input).cur.offset(1) })
                    || unsafe { *(*(*ctxt).input).cur.offset(1) } as i32 == 0
                {
                    break;
                }
                unsafe {
                    xmlParsePEReference(ctxt);
                }
            } else {
                if unsafe { !(*(*(*ctxt).input).cur as i32 == 0) } {
                    break;
                }
                if (safe_ctxt).inputNr <= 1 {
                    break;
                }
                unsafe { xmlPopInput_safe(ctxt) };
            }
            /*
             * Also increase the counter when entering or exiting a PERef.
             * The spec says: "When a parameter-entity reference is recognized
             * in the DTD and included, its replacement text MUST be enlarged
             * by the attachment of one leading and one following space (#x20)
             * character."
             */
            res += 1
        }
    }
    return res;
}
/* ***********************************************************************
*									*
*		Commodity functions to handle entities			*
*									*
************************************************************************/
/* *
* xmlPopInput:
* @ctxt:  an XML parser context
*
* xmlPopInput: the current input pointed by ctxt->input came to an end
*          pop it and return the next char.
*
* Returns the current xmlChar in the parser context
*/

pub fn xmlPopInput_parser(ctxt: xmlParserCtxtPtr) -> xmlChar {
    if ctxt.is_null() || unsafe { (*ctxt).inputNr <= 1 } {
        return 0;
    }
    let safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *__xmlParserDebugEntities() != 0 } {
        unsafe {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"Popping input %d\n\x00" as *const u8 as *const i8,
                (safe_ctxt).inputNr,
            );
        }
    }
    if (safe_ctxt).inputNr > 1
        && (safe_ctxt).inSubset == 0
        && (safe_ctxt).instate as i32 != XML_PARSER_EOF as i32
    {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"Unfinished entity outside the DTD\x00" as *const u8 as *const i8,
            );
        }
    }
    unsafe { xmlFreeInputStream_safe(unsafe { inputPop_parser(ctxt) }) };
    if unsafe { *(*(*ctxt).input).cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe((safe_ctxt).input, INPUT_CHUNK) };
    }
    return unsafe { *(*(*ctxt).input).cur };
}
/* *
* xmlPushInput:
* @ctxt:  an XML parser context
* @input:  an XML parser input fragment (entity, XML fragment ...).
*
* xmlPushInput: switch to a new input stream which is stacked on top
*               of the previous one(s).
* Returns -1 in case of error or the index in the input stack
*/

pub fn xmlPushInput(ctxt: xmlParserCtxtPtr, input: xmlParserInputPtr) -> i32 {
    let ret;
    let safe_ctxt = unsafe { &mut *ctxt };
    if input.is_null() {
        return -(1);
    }
    if unsafe { *__xmlParserDebugEntities() != 0 } {
        if !(safe_ctxt).input.is_null() && unsafe { !(*(*ctxt).input).filename.is_null() } {
            unsafe {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"%s(%d): \x00" as *const u8 as *const i8,
                    (*(*ctxt).input).filename,
                    (*(*ctxt).input).line,
                );
            }
        }
        unsafe {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"Pushing input %d : %.30s\n\x00" as *const u8 as *const i8,
                (safe_ctxt).inputNr + 1,
                (*input).cur,
            );
        }
    }
    if (safe_ctxt).inputNr > 40 && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
        || (safe_ctxt).inputNr > 1024
    {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
            loop {
                if !(1 < (safe_ctxt).inputNr) {
                    break;
                }
                xmlFreeInputStream_safe(inputPop_parser(ctxt));
            }
        }
        return -(1);
    }
    ret = unsafe { inputPush_safe(ctxt, input) };
    if (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32 {
        return -(1);
    }
    GROW(ctxt);
    return ret;
}
/* *
* xmlParseCharRef:
* @ctxt:  an XML parser context
*
* parse Reference declarations
*
* [66] CharRef ::= '&#' [0-9]+ ';' |
*                  '&#x' [0-9a-fA-F]+ ';'
*
* [ WFC: Legal Character ]
* Characters referred to using character references must match the
* production for Char.
*
* Returns the value parsed (as an int), 0 in case of error
*/

pub fn xmlParseCharRef(ctxt: xmlParserCtxtPtr) -> i32 {
    let mut val: i32 = 0;
    let mut count: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    /*
     * Using RAW/CUR/NEXT is okay since we are working on ASCII range here
     */
    if unsafe {
        *(*(*ctxt).input).cur == '&' as u8
            && *(*(*ctxt).input).cur.offset(1) == '#' as u8
            && *(*(*ctxt).input).cur.offset(2) == 'x' as u8
    } {
        SKIP(ctxt, 3);
        GROW(ctxt);
        loop {
            if unsafe { *(*(*ctxt).input).cur == ';' as u8 } {
                break;
            }
            /* loop blocked by count */
            if count > 20 {
                count = 0;
                GROW(ctxt);
                if (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32 {
                    return 0;
                }
            }
            count += 1;
            if unsafe { *(*(*ctxt).input).cur >= '0' as u8 && *(*(*ctxt).input).cur <= '9' as u8 } {
                val = val * 16 + unsafe { (*(*(*ctxt).input).cur - '0' as u8) as i32 };
            } else if unsafe {
                *(*(*ctxt).input).cur >= 'a' as u8 && *(*(*ctxt).input).cur <= 'f' as u8
            } && count < 20
            {
                val = unsafe { val * 16 + (*(*(*ctxt).input).cur - 'a' as u8) as i32 + 10 };
            } else if unsafe {
                *(*(*ctxt).input).cur >= 'A' as u8 && *(*(*ctxt).input).cur <= 'F' as u8
            } && count < 20
            {
                val = unsafe { val * 16 + (*(*(*ctxt).input).cur - 'A' as u8) as i32 + 10 }
            } else {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_INVALID_HEX_CHARREF, 0 as *const i8);
                }
                val = 0;
                break;
            }
            if val > 0x110000 {
                val = 0x110000
            }
            unsafe { xmlNextChar_safe(ctxt) };
            count += 1
        }
        if unsafe { *(*(*ctxt).input).cur == ';' as u8 } {
            /* on purpose to avoid reentrancy problems with NEXT and SKIP */
            unsafe {
                (*(*ctxt).input).col += 1;
                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(1)
            }
        }
    } else if unsafe {
        *(*(*ctxt).input).cur == '&' as u8 && *(*(*ctxt).input).cur.offset(1) == '#' as u8
    } {
        SKIP(ctxt, 2);
        GROW(ctxt);
        loop {
            if unsafe { *(*(*ctxt).input).cur == ';' as u8 } {
                break;
            }
            /* loop blocked by count */
            if count > 20 {
                count = 0;
                GROW(ctxt);
                if (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32 {
                    return 0;
                }
            }
            count += 1;
            if unsafe { *(*(*ctxt).input).cur >= '0' as u8 && *(*(*ctxt).input).cur <= '9' as u8 } {
                val = unsafe { val * 10 + (*(*(*ctxt).input).cur - '0' as u8) as i32 };
                if val > 0x110000 {
                    val = 0x110000
                }
                unsafe { xmlNextChar_safe(ctxt) };
                count += 1
            } else {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_INVALID_DEC_CHARREF, 0 as *const i8);
                }
                val = 0;
                break;
            }
        }
        if unsafe { *(*(*ctxt).input).cur == ';' as u8 } {
            /* on purpose to avoid reentrancy problems with NEXT and SKIP */
            unsafe {
                (*(*ctxt).input).col += 1;
                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(1)
            }
        }
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_INVALID_CHARREF, 0 as *const i8);
        }
    }
    /*
     * [ WFC: Legal Character ]
     * Characters referred to using character references must match the
     * production for Char.
     */
    if val >= 0x110000 {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"xmlParseCharRef: character reference out of bounds\n\x00" as *const u8
                    as *const i8,
                val,
            );
        }
    } else if IS_CHAR(val) {
        return val;
    } else {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"xmlParseCharRef: invalid xmlChar value %d\n\x00" as *const u8 as *const i8,
                val,
            );
        }
    }
    return 0;
}
/* *
* xmlParseStringCharRef:
* @ctxt:  an XML parser context
* @str:  a pointer to an index in the string
*
* parse Reference declarations, variant parsing from a string rather
* than an an input flow.
*
* [66] CharRef ::= '&#' [0-9]+ ';' |
*                  '&#x' [0-9a-fA-F]+ ';'
*
* [ WFC: Legal Character ]
* Characters referred to using character references must match the
* production for Char.
*
* Returns the value parsed (as an int), 0 in case of error, str will be
*         updated to the current value of the index
*/
fn xmlParseStringCharRef(ctxt: xmlParserCtxtPtr, str: *mut *const xmlChar) -> i32 {
    let mut ptr: *const xmlChar;
    let mut cur: xmlChar;
    let mut val: i32 = 0;
    if str.is_null() || unsafe { (*str).is_null() } {
        return 0;
    }
    unsafe {
        ptr = *str;
        cur = *ptr;
    }
    if cur == '&' as u8 && unsafe { *ptr.offset(1) == '#' as u8 && *ptr.offset(2) == 'x' as u8 } {
        unsafe {
            ptr = ptr.offset(3);
            cur = *ptr;
        }
        while cur != ';' as u8 {
            /* Non input consuming loop */
            if cur >= '0' as u8 && cur <= '9' as u8 {
                val = val * 16 + (cur - '0' as u8) as i32
            } else if cur >= 'a' as u8 && cur <= 'f' as u8 {
                val = val * 16 + (cur - 'a' as u8) as i32 + 10
            } else if cur >= 'A' as u8 && cur <= 'F' as u8 {
                val = val * 16 + (cur - 'A' as u8) as i32 + 10
            } else {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_INVALID_HEX_CHARREF, 0 as *const i8);
                }
                val = 0;
                break;
            }
            if val > 0x110000 {
                val = 0x110000
            }
            unsafe {
                ptr = ptr.offset(1);
                cur = *ptr;
            }
        }
        if cur == ';' as u8 {
            ptr = unsafe { ptr.offset(1) }
        }
    } else if cur == '&' as u8 && unsafe { *ptr.offset(1) == '#' as u8 } {
        unsafe {
            ptr = ptr.offset(2);
            cur = *ptr;
        }
        while cur != ';' as u8 {
            /* Non input consuming loops */
            if cur >= '0' as u8 && cur <= '9' as u8 {
                val = val * 10 + (cur - '0' as u8) as i32;
                if val > 0x110000 {
                    val = 0x110000
                }
                unsafe {
                    ptr = ptr.offset(1);
                    cur = *ptr
                }
            } else {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_INVALID_DEC_CHARREF, 0 as *const i8);
                }
                val = 0;
                break;
            }
        }
        if cur == ';' as u8 {
            ptr = unsafe { ptr.offset(1) }
        }
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_INVALID_CHARREF, 0 as *const i8);
        }
        return 0;
    }
    unsafe { *str = ptr };
    /*
     * [ WFC: Legal Character ]
     * Characters referred to using character references must match the
     * production for Char.
     */
    if val >= 0x110000 {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"xmlParseStringCharRef: character reference out of bounds\n\x00" as *const u8
                    as *const i8,
                val,
            );
        }
    } else if IS_CHAR(val) {
        return val;
    } else {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"xmlParseStringCharRef: invalid xmlChar value %d\n\x00" as *const u8 as *const i8,
                val,
            );
        }
    }
    return 0;
}
/* *
* xmlParserHandlePEReference:
* @ctxt:  the parser context
*
* [69] PEReference ::= '%' Name ';'
*
* [ WFC: No Recursion ]
* A parsed entity must not contain a recursive
* reference to itself, either directly or indirectly.
*
* [ WFC: Entity Declared ]
* In a document without any DTD, a document with only an internal DTD
* subset which contains no parameter entity references, or a document
* with "standalone='yes'", ...  ... The declaration of a parameter
* entity must precede any reference to it...
*
* [ VC: Entity Declared ]
* In a document with an external subset or external parameter entities
* with "standalone='no'", ...  ... The declaration of a parameter entity
* must precede any reference to it...
*
* [ WFC: In DTD ]
* Parameter-entity references may only appear in the DTD.
* NOTE: misleading but this is handled.
*
* A PEReference may have been detected in the current input stream
* the handling is done accordingly to
*      http://www.w3.org/TR/REC-xml#entproc
* i.e.
*   - Included in literal in entity values
*   - Included as Parameter Entity reference within DTDs
*/
pub fn xmlParserHandlePEReference(ctxt: xmlParserCtxtPtr) {
    let safe_ctxt = unsafe { &mut *ctxt };
    match (safe_ctxt).instate as i32 {
        XML_PARSER_CDATA_SECTION
        | XML_PARSER_COMMENT
        | XML_PARSER_START_TAG
        | XML_PARSER_END_TAG => return,
        XML_PARSER_EOF => {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_PEREF_AT_EOF, 0 as *const i8);
            }
            return;
        }
        XML_PARSER_PROLOG | XML_PARSER_START | XML_PARSER_MISC => {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_PEREF_IN_PROLOG, 0 as *const i8);
            }
            return;
        }
        XML_PARSER_ENTITY_DECL
        | XML_PARSER_CONTENT
        | XML_PARSER_ATTRIBUTE_VALUE
        | XML_PARSER_PI
        | XML_PARSER_SYSTEM_LITERAL
        | XML_PARSER_PUBLIC_LITERAL => {
            /* we just ignore it there */
            return;
        }
        XML_PARSER_EPILOG => {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_PEREF_IN_EPILOG, 0 as *const i8);
            }
            return;
        }
        XML_PARSER_ENTITY_VALUE => {
            /*
             * NOTE: in the case of entity values, we don't do the
             *       substitution here since we need the literal
             *       entity value to be able to save the internal
             *       subset of the document.
             *       This will be handled by xmlStringDecodeEntities
             */
            return;
        }
        XML_PARSER_DTD => {
            /*
             * [WFC: Well-Formedness Constraint: PEs in Internal Subset]
             * In the internal DTD subset, parameter-entity references
             * can occur only where markup declarations can occur, not
             * within markup declarations.
             * In that case this is handled in xmlParseMarkupDecl
             */
            if (safe_ctxt).external == 0 && (safe_ctxt).inputNr == 1 {
                return;
            }
            if unsafe {
                *(*(*ctxt).input).cur.offset(1) as i32 == 0x20 as i32
                    || 0x9 as i32 <= *(*(*ctxt).input).cur.offset(1) as i32
                        && *(*(*ctxt).input).cur.offset(1) as i32 <= 0xa as i32
                    || *(*(*ctxt).input).cur.offset(1) as i32 == 0xd as i32
                    || *(*(*ctxt).input).cur.offset(1) as i32 == 0
            } {
                return;
            }
        }
        XML_PARSER_IGNORE => return,
        _ => {}
    }
    unsafe {
        xmlParsePEReference(ctxt);
    }
}
/*
* Macro used to grow the current buffer.
* buffer##_size is expected to be a size_t
* mem_error: is expected to handle memory allocation failures
*/
/* *
* xmlStringLenDecodeEntities:
* @ctxt:  the parser context
* @str:  the input string
* @len: the string length
* @what:  combination of XML_SUBSTITUTE_REF and XML_SUBSTITUTE_PEREF
* @end:  an end marker xmlChar, 0 if none
* @end2:  an end marker xmlChar, 0 if none
* @end3:  an end marker xmlChar, 0 if none
*
* Takes a entity string content and process to do the adequate substitutions.
*
* [67] Reference ::= EntityRef | CharRef
*
* [69] PEReference ::= '%' Name ';'
*
* Returns A newly allocated string with the substitution done. The caller
*      must deallocate it !
*/

const XML_PARSER_BUFFER_SIZE: u64 = 100;
const XML_SUBSTITUTE_REF: i32 = 1;
pub fn xmlStringLenDecodeEntities(
    ctxt: xmlParserCtxtPtr,
    mut str: *const xmlChar,
    len: i32,
    what: i32,
    end: xmlChar,
    end2: xmlChar,
    end3: xmlChar,
) -> *mut xmlChar {
    let mut current_block: u64;
    let mut buffer: *mut xmlChar = 0 as *mut xmlChar;
    let mut buffer_size: size_t = 0;
    let mut nbchars: size_t = 0;
    let mut current: *mut xmlChar = 0 as *mut xmlChar;
    let mut rep: *mut xmlChar = 0 as *mut xmlChar;
    let mut last: *const xmlChar;
    let mut ent: xmlEntityPtr;
    let mut c: i32;
    let mut l: i32 = 0;
    if ctxt.is_null() || str.is_null() || len < 0 {
        return 0 as *mut xmlChar;
    }
    let safe_ctxt = unsafe { &mut *ctxt };
    last = unsafe { str.offset(len as isize) };
    if (safe_ctxt).depth > 40 && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
        || (safe_ctxt).depth > 1024
    {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    /*
     * allocate a translation buffer.
     */
    buffer_size = XML_PARSER_BIG_BUFFER_SIZE;
    buffer = unsafe { xmlMallocAtomic_safe(buffer_size) } as *mut xmlChar;
    if buffer.is_null() {
        current_block = 1;
    } else {
        /*
         * OK loop until we reach one of the ending char or a size limit.
         * we are operating on already parsed values.
         */
        if str < last {
            unsafe {
                c = xmlStringCurrentChar(ctxt, str, &mut l);
            }
        } else {
            c = 0
        }
        loop {
            if !(c != 0
                && c != end as i32
                && c != end2 as i32
                && c != end3 as i32
                && (safe_ctxt).instate as i32 != XML_PARSER_EOF as i32)
            {
                current_block = 2;
                break;
            }
            if c == 0 {
                current_block = 2;
                break;
            }
            if c == '&' as i32 && unsafe { *str.offset(1) == '#' as u8 } {
                let val: i32 = xmlParseStringCharRef(ctxt, &mut str);
                if val == 0 {
                    current_block = 7451279748152143041;
                    break;
                }
                if 0 == 1 {
                    unsafe { *buffer.offset(nbchars as isize) = val as xmlChar }
                    nbchars += 1;
                } else {
                    nbchars += unsafe {
                        xmlCopyCharMultiByte(&mut *buffer.offset(nbchars as isize), val) as size_t
                    };
                }
                if nbchars + XML_PARSER_BUFFER_SIZE > buffer_size {
                    let mut tmp: *mut xmlChar;
                    let new_size: size_t = (buffer_size * 2) + XML_PARSER_BUFFER_SIZE;
                    if new_size < buffer_size {
                        current_block = 1;
                        break;
                    }
                    tmp = unsafe { xmlRealloc_safe(buffer as *mut (), new_size) } as *mut xmlChar;
                    if tmp.is_null() {
                        current_block = 1;
                        break;
                    }
                    buffer = tmp;
                    buffer_size = new_size
                }
            } else if c == '&' as i32 && what & XML_SUBSTITUTE_REF != 0 {
                if unsafe { *__xmlParserDebugEntities() != 0 } {
                    unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"String decoding Entity Reference: %.30s\n\x00" as *const u8
                                as *const i8,
                            str,
                        );
                    }
                }
                unsafe {
                    ent = xmlParseStringEntityRef(ctxt, &mut str);
                    unsafe { xmlParserEntityCheck(ctxt, 0, ent, 0) };
                }
                if !ent.is_null() {
                    (safe_ctxt).nbentities += unsafe { ((*ent).checked / 2) as u64 };
                }
                if !ent.is_null()
                    && unsafe { (*ent).etype as u32 == XML_INTERNAL_PREDEFINED_ENTITY as u32 }
                {
                    if unsafe { !(*ent).content.is_null() } {
                        if 0 == 1 {
                            let fresh30 = nbchars;
                            nbchars += 1;
                            unsafe {
                                *buffer.offset(fresh30 as isize) = *(*ent).content.offset(0);
                            }
                        } else {
                            nbchars += unsafe {
                                xmlCopyCharMultiByte(
                                    &mut *buffer.offset(nbchars as isize),
                                    *(*ent).content.offset(0) as i32,
                                ) as size_t
                            }
                        }
                        if nbchars + XML_PARSER_BUFFER_SIZE > buffer_size {
                            let mut tmp_0: *mut xmlChar;
                            let new_size_0: size_t = (buffer_size * 2) + XML_PARSER_BUFFER_SIZE;
                            if new_size_0 < buffer_size {
                                current_block = 1;
                                break;
                            }
                            tmp_0 = unsafe { xmlRealloc_safe(buffer as *mut (), new_size_0) }
                                as *mut xmlChar;
                            if tmp_0.is_null() {
                                current_block = 1;
                                break;
                            }
                            buffer = tmp_0;
                            buffer_size = new_size_0
                        }
                    } else {
                        unsafe {
                            xmlFatalErrMsg(
                                ctxt,
                                XML_ERR_INTERNAL_ERROR,
                                b"predefined entity has no content\n\x00" as *const u8 as *const i8,
                            );
                        }
                        current_block = 7451279748152143041;
                        break;
                    }
                } else if !ent.is_null() && unsafe { !(*ent).content.is_null() } {
                    (safe_ctxt).depth += 1;
                    rep = unsafe { xmlStringDecodeEntities(ctxt, (*ent).content, what, 0, 0, 0) };
                    (safe_ctxt).depth -= 1;
                    if rep.is_null() {
                        unsafe {
                            *(*ent).content.offset(0) = 0;
                        }
                        current_block = 7451279748152143041;
                        break;
                    } else {
                        current = rep;
                        while unsafe { *current } as i32 != 0 {
                            /* non input consuming loop */
                            let tmp_current = current;
                            current = unsafe { current.offset(1) };
                            let tmp_nbchars = nbchars;
                            nbchars += 1;
                            unsafe { *buffer.offset(tmp_nbchars as isize) = *tmp_current };
                            if !(nbchars + XML_PARSER_BUFFER_SIZE > buffer_size) {
                                continue;
                            }
                            if unsafe { xmlParserEntityCheck(ctxt, nbchars, ent, 0) } != 0 {
                                current_block = 7451279748152143041;
                                break;
                            }
                            let mut tmp_1: *mut xmlChar;
                            let new_size_1: size_t = (buffer_size * 2) + XML_PARSER_BUFFER_SIZE;
                            if new_size_1 < buffer_size {
                                current_block = 1;
                                break;
                            }
                            tmp_1 = unsafe { xmlRealloc_safe(buffer as *mut (), new_size_1) }
                                as *mut xmlChar;
                            if tmp_1.is_null() {
                                current_block = 1;
                                break;
                            }
                            buffer = tmp_1;
                            buffer_size = new_size_1
                        }
                        unsafe { xmlFree_safe(rep as *mut ()) };
                        rep = 0 as *mut xmlChar
                    }
                } else if !ent.is_null() {
                    let mut i: i32 = unsafe { xmlStrlen_safe(unsafe { (*ent).name }) };
                    let mut cur: *const xmlChar = unsafe { (*ent).name };
                    let fresh33 = nbchars;
                    nbchars += 1;
                    unsafe {
                        *buffer.offset(fresh33 as isize) = '&' as xmlChar;
                    }
                    if nbchars + i as u64 + XML_PARSER_BUFFER_SIZE > buffer_size {
                        let mut tmp_2: *mut xmlChar = 0 as *mut xmlChar;
                        let new_size_2: size_t =
                            buffer_size * 2 + i as u64 + XML_PARSER_BUFFER_SIZE;
                        if new_size_2 < buffer_size {
                            current_block = 1;
                            break;
                        }
                        tmp_2 = unsafe { xmlRealloc_safe(buffer as *mut (), new_size_2) }
                            as *mut xmlChar;
                        if tmp_2.is_null() {
                            current_block = 1;
                            break;
                        }
                        buffer = tmp_2;
                        buffer_size = new_size_2
                    }
                    while i > 0 {
                        let temp_cur = cur;
                        cur = unsafe { cur.offset(1) };
                        unsafe { *buffer.offset(nbchars as isize) = *temp_cur };
                        nbchars += 1;
                        i -= 1
                    }
                    unsafe { *buffer.offset(nbchars as isize) = ';' as xmlChar }
                    nbchars += 1;
                }
            } else if c == '%' as i32 && what & 2 != 0 {
                if unsafe { *__xmlParserDebugEntities() != 0 } {
                    unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"String decoding PE Reference: %.30s\n\x00" as *const u8 as *const i8,
                            str,
                        );
                    }
                }
                ent = unsafe { xmlParseStringPEReference(ctxt, &mut str) };
                unsafe { xmlParserEntityCheck(ctxt, 0, ent, 0) };
                if !ent.is_null() {
                    (safe_ctxt).nbentities += unsafe { ((*ent).checked / 2) as u64 };
                }
                if !ent.is_null() {
                    if unsafe { (*ent).content.is_null() } {
                        /*
                         * Note: external parsed entities will not be loaded,
                         * it is not required for a non-validating parser to
                         * complete external PEReferences coming from the
                         * internal subset
                         */
                        if (safe_ctxt).options & XML_PARSE_NOENT as i32 != 0
                            || (safe_ctxt).options & XML_PARSE_DTDVALID as i32 != 0
                            || (safe_ctxt).validate != 0
                        {
                            xmlLoadEntityContent(ctxt, ent);
                        } else {
                            unsafe {
                                xmlWarningMsg(
                                    ctxt,
                                    XML_ERR_ENTITY_PROCESSING,
                                    b"not validating will not read content for PE entity %s\n\x00"
                                        as *const u8
                                        as *const i8,
                                    (*ent).name,
                                    0 as *const xmlChar,
                                );
                            }
                        }
                    }
                    (safe_ctxt).depth += 1;
                    rep = unsafe { xmlStringDecodeEntities(ctxt, (*ent).content, what, 0, 0, 0) };

                    (safe_ctxt).depth -= 1;
                    if rep.is_null() {
                        if unsafe { !(*ent).content.is_null() } {
                            unsafe { *(*ent).content.offset(0) = 0 }
                        }
                        current_block = 7451279748152143041;
                        break;
                    } else {
                        current = rep;
                        while unsafe { *current as i32 != 0 } {
                            /* non input consuming loop */
                            let temp_current = current;
                            current = unsafe { current.offset(1) };
                            unsafe { *buffer.offset(nbchars as isize) = *temp_current };
                            nbchars += 1;
                            if !(nbchars + XML_PARSER_BUFFER_SIZE > buffer_size) {
                                continue;
                            }
                            if xmlParserEntityCheck(ctxt, nbchars, ent, 0) != 0 {
                                current_block = 7451279748152143041;
                                break;
                            }
                            let mut tmp_3: *mut xmlChar;
                            let new_size_3: size_t = buffer_size * 2 + 100;
                            if new_size_3 < buffer_size {
                                current_block = 1;
                                break;
                            }
                            tmp_3 = unsafe { xmlRealloc_safe(buffer as *mut (), new_size_3) }
                                as *mut xmlChar;
                            if tmp_3.is_null() {
                                current_block = 1;
                                break;
                            }
                            buffer = tmp_3;
                            buffer_size = new_size_3
                        }
                        unsafe { xmlFree_safe(rep as *mut ()) };
                        rep = 0 as *mut xmlChar
                    }
                }
            } else {
                if l == 1 {
                    unsafe { *buffer.offset(nbchars as isize) = c as xmlChar }
                    nbchars += 1;
                } else {
                    nbchars = unsafe {
                        xmlCopyCharMultiByte(&mut *buffer.offset(nbchars as isize), c) as size_t
                    }
                }
                str = unsafe { str.offset(l as isize) };
                if nbchars + XML_PARSER_BUFFER_SIZE > buffer_size {
                    let mut tmp_4: *mut xmlChar = 0 as *mut xmlChar;
                    let new_size_4: size_t = buffer_size * 2 + 100;
                    if new_size_4 < buffer_size {
                        current_block = 1;
                        break;
                    }
                    tmp_4 =
                        unsafe { xmlRealloc_safe(buffer as *mut (), new_size_4) } as *mut xmlChar;
                    if tmp_4.is_null() {
                        current_block = 1;
                        break;
                    }
                    buffer = tmp_4;
                    buffer_size = new_size_4
                }
            }
            if str < last {
                c = xmlStringCurrentChar(ctxt, str, &mut l)
            } else {
                c = 0
            }
        }
        match current_block {
            1 => {}
            7451279748152143041 => {}
            _ => {
                unsafe {
                    *buffer.offset(nbchars as isize) = 0;
                }
                return buffer;
            }
        }
    }
    match current_block {
        1 => unsafe {
            xmlErrMemory(ctxt, 0 as *const i8);
        },
        _ => {}
    }
    if !rep.is_null() {
        unsafe { xmlFree_safe(rep as *mut ()) };
    }
    if !buffer.is_null() {
        unsafe { xmlFree_safe(buffer as *mut ()) };
    }
    return 0 as *mut xmlChar;
}
/* *
* xmlStringDecodeEntities:
* @ctxt:  the parser context
* @str:  the input string
* @what:  combination of XML_SUBSTITUTE_REF and XML_SUBSTITUTE_PEREF
* @end:  an end marker xmlChar, 0 if none
* @end2:  an end marker xmlChar, 0 if none
* @end3:  an end marker xmlChar, 0 if none
*
* Takes a entity string content and process to do the adequate substitutions.
*
* [67] Reference ::= EntityRef | CharRef
*
* [69] PEReference ::= '%' Name ';'
*
* Returns A newly allocated string with the substitution done. The caller
*      must deallocate it !
*/

pub fn xmlStringDecodeEntities(
    ctxt: xmlParserCtxtPtr,
    str: *const xmlChar,
    what: i32,
    end: xmlChar,
    end2: xmlChar,
    end3: xmlChar,
) -> *mut xmlChar {
    if ctxt.is_null() || str.is_null() {
        return 0 as *mut xmlChar;
    }
    return xmlStringLenDecodeEntities(
        ctxt,
        str,
        unsafe { xmlStrlen_safe(str) },
        what,
        end,
        end2,
        end3,
    );
}
/* ***********************************************************************
*									*
*		Commodity functions, cleanup needed ?			*
*									*
************************************************************************/
/* *
* areBlanks:
* @ctxt:  an XML parser context
* @str:  a xmlChar *
* @len:  the size of @str
* @blank_chars: we know the chars are blanks
*
* Is this a sequence of blank chars that one can ignore ?
*
* Returns 1 if ignorable 0 otherwise.
*/
fn areBlanks(ctxt: xmlParserCtxtPtr, str: *const xmlChar, len: i32, blank_chars: i32) -> i32 {
    let mut i: i32;
    let mut ret: i32;
    let mut lastChild: xmlNodePtr;
    /*
     * Don't spend time trying to differentiate them, the same callback is
     * used !
     */
    if unsafe { (*(*ctxt).sax).ignorableWhitespace == (*(*ctxt).sax).characters } {
        return 0;
    }
    let safe_ctxt = unsafe { &mut *ctxt };
    /*
     * Check for xml:space value.
     */
    if unsafe { (*ctxt).space.is_null() || *(*ctxt).space == 1 || *(*ctxt).space == -(2) } {
        return 0;
    }
    /*
     * Check that the string is made of blanks
     */
    if blank_chars == 0 {
        i = 0;
        while i < len {
            if unsafe { !IS_BLANK_CH(str) } {
                return 0;
            }
            i += 1
        }
    }
    /*
     * Look if the element is mixed content in the DTD if available
     */
    if (safe_ctxt).node.is_null() {
        return 0;
    }
    if !(safe_ctxt).myDoc.is_null() {
        ret = unsafe { xmlIsMixedElement((safe_ctxt).myDoc, (*(*ctxt).node).name) };
        if ret == 0 {
            return 1;
        }
        if ret == 1 {
            return 0;
        }
    }
    /*
     * Otherwise, heuristic :-\
     */
    if unsafe { *(*(*ctxt).input).cur != '<' as u8 && *(*(*ctxt).input).cur as i32 != 0xd } {
        return 0;
    }
    if unsafe {
        (*(*ctxt).node).children.is_null()
            && *(*(*ctxt).input).cur == '<' as u8
            && *(*(*ctxt).input).cur.offset(1) == '/' as u8
    } {
        return 0;
    }
    lastChild = unsafe { xmlGetLastChild_safe((safe_ctxt).node as *const xmlNode) };
    if lastChild.is_null() {
        if unsafe {
            (*(*ctxt).node).type_0 as u32 != XML_ELEMENT_NODE as u32
                && !(*(*ctxt).node).content.is_null()
        } {
            return 0;
        }
    } else if unsafe { xmlNodeIsText_safe(lastChild as *const xmlNode) } != 0 {
        return 0;
    } else {
        if unsafe {
            !(*(*ctxt).node).children.is_null() && xmlNodeIsText((*(*ctxt).node).children) != 0
        } {
            return 0;
        }
    }
    return 1;
}
/* ***********************************************************************
*									*
*		Extra stuff for namespace support			*
*	Relates to http://www.w3.org/TR/WD-xml-names			*
*									*
 ************************************************************************/
/* *
* xmlSplitQName:
* @ctxt:  an XML parser context
* @name:  an XML parser context
* @prefix:  a xmlChar **
*
* parse an UTF8 encoded XML qualified name string
*
* [NS 5] QName ::= (Prefix ':')? LocalPart
*
* [NS 6] Prefix ::= NCName
*
* [NS 7] LocalPart ::= NCName
*
* Returns the local part, and prefix is updated
*   to get the Prefix if any.
*/
const XML_MAX_NAMELEN: usize = 100;
pub fn xmlSplitQName(
    ctxt: xmlParserCtxtPtr,
    name: *const xmlChar,
    prefix: *mut *mut xmlChar,
) -> *mut xmlChar {
    let mut buf: [xmlChar; XML_MAX_NAMELEN + 5] = [0; XML_MAX_NAMELEN + 5];
    let mut buffer: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut max: i32 = XML_MAX_NAMELEN as i32;
    let mut ret: *mut xmlChar = 0 as *mut xmlChar;
    let mut cur: *const xmlChar = name;
    let mut c: i32;
    if prefix.is_null() {
        return 0 as *mut xmlChar;
    }
    unsafe { *prefix = 0 as *mut xmlChar };
    if cur.is_null() {
        return 0 as *mut xmlChar;
    }

    match () {
        #[cfg(HAVE_parser_XML_XML_NAMESPACE)]
        _ => {}
        #[cfg(not(HAVE_parser_XML_XML_NAMESPACE))]
        _ => {
            if unsafe {
                *cur.offset(0) == 'x' as u8
                    && *cur.offset(1) == 'm' as u8
                    && *cur.offset(3) == ':' as u8
                    && *cur.offset(3) == ':' as u8
            } {
                return unsafe { xmlStrdup_safe(name) };
            }
        }
    };

    /* nasty but well=formed */
    if unsafe { *cur.offset(0) == ':' as u8 } {
        return unsafe { xmlStrdup_safe(name) };
    }
    unsafe {
        c = *cur as i32;
        cur = cur.offset(1);
    }
    while c != 0 && c != ':' as i32 && len < max {
        /* tested bigname.xml */
        buf[len as usize] = c as xmlChar;
        len = len + 1;
        unsafe {
            c = *cur as i32;
            cur = cur.offset(1);
        }
    }
    if len >= max {
        /*
         * Okay someone managed to make a huge name, so he's ready to pay
         * for the processing speed.
         */
        max = len * 2;
        buffer =
            unsafe { xmlMallocAtomic_safe((max as u64).wrapping_mul(size_of::<xmlChar>() as u64)) }
                as *mut xmlChar;
        if buffer.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return 0 as *mut xmlChar;
        }
        unsafe { memcpy_safe(buffer as *mut (), buf.as_mut_ptr() as *const (), len as u64) };
        while c != 0 && c != ':' as i32 {
            /* tested bigname.xml */
            if len + 10 > max {
                let tmp: *mut xmlChar;
                max *= 2;
                tmp = unsafe {
                    xmlRealloc_safe(buffer as *mut (), max as u64 * size_of::<xmlChar>() as u64)
                } as *mut xmlChar;
                if tmp.is_null() {
                    unsafe { xmlFree_safe(buffer as *mut ()) };
                    unsafe {
                        xmlErrMemory(ctxt, 0 as *const i8);
                    }
                    return 0 as *mut xmlChar;
                }
                buffer = tmp
            }
            unsafe {
                *buffer.offset(len as isize) = c as xmlChar;
            }
            len = len + 1;
            unsafe {
                c = *cur as i32;
                cur = cur.offset(1);
            }
        }
        unsafe { *buffer.offset(len as isize) = 0 }
    }
    if c == ':' as i32 && unsafe { *cur as i32 == 0 } {
        if !buffer.is_null() {
            unsafe { xmlFree_safe(buffer as *mut ()) };
        }
        unsafe {
            *prefix = 0 as *mut xmlChar;
        }
        return unsafe { xmlStrdup_safe(name) };
    }
    if buffer.is_null() {
        ret = unsafe { xmlStrndup_safe(buf.as_mut_ptr(), len) }
    } else {
        ret = buffer;
        buffer = 0 as *mut xmlChar;
        max = 100
    }
    if c == ':' as i32 {
        unsafe {
            c = *cur as i32;
            *prefix = ret;
        }
        if c == 0 {
            return unsafe {
                xmlStrndup_safe(b"\x00" as *const u8 as *const i8 as *mut xmlChar, 0)
            };
        }
        len = 0;
        /*
         * Check that the first character is proper to start
         * a new name
         */
        if !(c >= 0x61 && c <= 0x7a || c >= 0x41 && c <= 0x5a || c == '_' as i32 || c == ':' as i32)
        {
            let mut l: i32 = 0;
            let first: i32 = unsafe { xmlStringCurrentChar(ctxt, cur, &mut l) };
            if !((if first < 0x100 {
                (0x41 <= first && first <= 0x5a
                    || 0x61 <= first && first <= 0x7a
                    || 0xc0 <= first && first <= 0xd6
                    || 0xd8 <= first && first <= 0xf6
                    || 0xf8 <= first) as i32
            } else {
                unsafe { xmlCharInRange_safe(first as u32, unsafe { &xmlIsBaseCharGroup }) }
            }) != 0
                || (if first < 0x100 {
                    0
                } else {
                    (0x4e00 <= first && first <= 0x9fa5
                        || first == 0x3007
                        || 0x3021 <= first && first <= 0x3029) as i32
                }) != 0)
                && first != '_' as i32
            {
                unsafe {
                    xmlFatalErrMsgStr(
                        ctxt,
                        XML_NS_ERR_QNAME,
                        b"Name %s is not XML Namespace compliant\n\x00" as *const u8 as *const i8,
                        name,
                    );
                }
            }
        }
        cur = unsafe { cur.offset(1) };
        while c != 0 && len < max {
            /* tested bigname2.xml */
            buf[len as usize] = c as xmlChar;
            len = len + 1;
            unsafe {
                c = *cur as i32;
                cur = cur.offset(1);
            }
        }
        if len >= max {
            /*
             * Okay someone managed to make a huge name, so he's ready to pay
             * for the processing speed.
             */
            max = len * 2;
            buffer = unsafe {
                xmlMallocAtomic_safe((max as u64).wrapping_mul(size_of::<xmlChar>() as u64))
            } as *mut xmlChar;
            if buffer.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                return 0 as *mut xmlChar;
            }
            unsafe { memcpy_safe(buffer as *mut (), buf.as_mut_ptr() as *const (), len as u64) };
            while c != 0 {
                /* tested bigname2.xml */
                if len + 10 > max {
                    let mut tmp_0: *mut xmlChar;
                    max *= 2;
                    tmp_0 = unsafe {
                        xmlRealloc_safe(buffer as *mut (), max as u64 * size_of::<xmlChar>() as u64)
                    } as *mut xmlChar;
                    if tmp_0.is_null() {
                        unsafe {
                            xmlErrMemory(ctxt, 0 as *const i8);
                        }
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                    buffer = tmp_0
                }
                unsafe {
                    *buffer.offset(len as isize) = c as xmlChar;
                    len = len + 1;
                }
                unsafe {
                    c = *cur as i32;
                    cur = cur.offset(1);
                }
            }
            unsafe { *buffer.offset(len as isize) = 0 }
        }
        if buffer.is_null() {
            ret = unsafe { xmlStrndup_safe(buf.as_mut_ptr(), len) }
        } else {
            ret = buffer
        }
    }
    return ret;
}
/* ***********************************************************************
*									*
*			The parser itself				*
*	Relates to http://www.w3.org/TR/REC-xml				*
*									*
 ************************************************************************/
/* ***********************************************************************
*									*
*	Routines to parse Name, NCName and NmToken			*
*									*
************************************************************************/

#[cfg(HAVE_parser_DEBUG)]
const nbParseName: i64 = 0;
#[cfg(HAVE_parser_DEBUG)]
const nbParseNmToken: i64 = 0;
#[cfg(HAVE_parser_DEBUG)]
const nbParseNCName: i64 = 0;
#[cfg(HAVE_parser_DEBUG)]
const nbParseNCNameComplex: i64 = 0;
#[cfg(HAVE_parser_DEBUG)]
const nbParseNameComplex: i64 = 0;
#[cfg(HAVE_parser_DEBUG)]
const nbParseStringName: i64 = 0;

/*
* The two following functions are related to the change of accepted
* characters for Name and NmToken in the Revision 5 of XML-1.0
* They correspond to the modified production [4] and the new production [4a]
* changes in that revision. Also note that the macros used for the
* productions Letter, Digit, CombiningChar and Extender are not needed
* anymore.
* We still keep compatibility to pre-revision5 parsing semantic if the
* new XML_PARSE_OLD10 option is given to the parser.
*/
fn xmlIsNameStartChar(ctxt: xmlParserCtxtPtr, c: i32) -> i32 {
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).options & XML_PARSE_OLD10 as i32 == 0 {
        /*
         * Use the new checks of production [4] [4a] amd [5] of the
         * Update 5 of XML-1.0
         */
        if c != ' ' as i32
            && c != '>' as i32
            && c != '/' as i32
            && (c >= 'a' as i32 && c <= 'z' as i32
                || c >= 'A' as i32 && c <= 'Z' as i32
                || c == '_' as i32
                || c == ':' as i32
                || c >= 0xc0 && c <= 0xd6
                || c >= 0xd8 && c <= 0xf6
                || c >= 0xf8 && c <= 0x2ff
                || c >= 0x370 && c <= 0x37d
                || c >= 0x37f && c <= 0x1fff
                || c >= 0x200c && c <= 0x200d
                || c >= 0x2070 && c <= 0x218f
                || c >= 0x2c00 && c <= 0x2fef
                || c >= 0x3001 && c <= 0xd7ff
                || c >= 0xf900 && c <= 0xfdcf
                || c >= 0xfdf0 && c <= 0xfffd
                || c >= 0x10000 && c <= 0xeffff)
        {
            return 1;
        }
    } else {
        if (IS_LETTER(c, unsafe { &xmlIsBaseCharGroup }) || c == '_' as i32 || c == ':' as i32) {
            return 1;
        }
    }
    return 0;
}
fn xmlIsNameChar(ctxt: xmlParserCtxtPtr, c: i32) -> i32 {
    let safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).options & XML_PARSE_OLD10 as i32 == 0 {
        /*
         * Use the new checks of production [4] [4a] amd [5] of the
         * Update 5 of XML-1.0
         */
        if c != ' ' as i32
            && c != '>' as i32
            && c != '/' as i32
            && (c >= 'a' as i32 && c <= 'z' as i32
                || c >= 'A' as i32 && c <= 'Z' as i32
                || c >= '0' as i32 && c <= '9' as i32
                || c == '_' as i32
                || c == ':' as i32
                || c == '-' as i32
                || c == '.' as i32
                || c == 0xb7
                || c >= 0xc0 && c <= 0xd6
                || c >= 0xd8 && c <= 0xf6
                || c >= 0xf8 && c <= 0x2ff
                || c >= 0x300 && c <= 0x36f
                || c >= 0x370 && c <= 0x37d
                || c >= 0x37f && c <= 0x1fff
                || c >= 0x200c && c <= 0x200d
                || c >= 0x203f && c <= 0x2040
                || c >= 0x2070 && c <= 0x218f
                || c >= 0x2c00 && c <= 0x2fef
                || c >= 0x3001 && c <= 0xd7ff
                || c >= 0xf900 && c <= 0xfdcf
                || c >= 0xfdf0 && c <= 0xfffd
                || c >= 0x10000 && c <= 0xeffff)
        {
            return 1;
        }
    } else {
        if IS_LETTER(c, unsafe { &xmlIsBaseCharGroup })
            || IS_DIGIT(c, unsafe { &xmlIsBaseCharGroup })
            || c == '.' as i32
            || c == '-' as i32
            || c == '_' as i32
            || c == ':' as i32
            || IS_COMBINING(c, unsafe { &xmlIsBaseCharGroup })
            || IS_EXTENDER(c, unsafe { &xmlIsBaseCharGroup })
        {
            return 1;
        }
    }
    return 0;
}

const XML_PARSER_CHUNK_SIZE: i32 = 100;
const XML_MAX_NAME_LENGTH: i32 = 50000;
fn xmlParseNameComplex(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let safe_ctxt = unsafe { &mut *ctxt };
    let mut len: i32 = 0;
    let mut l: i32 = 0;
    let mut c: i32 = 0;
    let mut count: i32 = 0;

    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseNameComplex = nbParseNameComplex + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    /*
     * Handler for more complex cases
     */
    GROW(ctxt);
    c = unsafe { xmlCurrentChar(ctxt, &mut l) };
    if (safe_ctxt).options & XML_PARSE_OLD10 as i32 == 0 {
        /*
         * Use the new checks of production [4] [4a] amd [5] of the
         * Update 5 of XML-1.0
         */
        if c == ' ' as i32
            || c == '>' as i32
            || c == '/' as i32
            || !(c >= 'a' as i32 && c <= 'z' as i32
                || c >= 'A' as i32 && c <= 'Z' as i32
                || c == '_' as i32
                || c == ':' as i32
                || c >= 0xc0 && c <= 0xd6
                || c >= 0xd8 && c <= 0xf6
                || c >= 0xf8 && c <= 0x2ff
                || c >= 0x370 && c <= 0x37d
                || c >= 0x37f && c <= 0x1fff
                || c >= 0x200c && c <= 0x200d
                || c >= 0x2070 && c <= 0x218f
                || c >= 0x2c00 && c <= 0x2fef
                || c >= 0x3001 && c <= 0xd7ff
                || c >= 0xf900 && c <= 0xfdcf
                || c >= 0xfdf0 && c <= 0xfffd
                || c >= 0x10000 && c <= 0xeffff)
        {
            return 0 as *const xmlChar;
        }
        len += l;
        NEXTL(ctxt, l);
        unsafe {
            c = xmlCurrentChar(ctxt, &mut l);
        }
        while c != ' ' as i32
            && c != '>' as i32
            && c != '/' as i32
            && (c >= 'a' as i32 && c <= 'z' as i32
                || c >= 'A' as i32 && c <= 'Z' as i32
                || c >= '0' as i32 && c <= '9' as i32
                || c == '_' as i32
                || c == ':' as i32
                || c == '-' as i32
                || c == '.' as i32
                || c == 0xb7
                || c >= 0xc0 && c <= 0xd6
                || c >= 0xd8 && c <= 0xf6
                || c >= 0xf8 && c <= 0x2ff
                || c >= 0x300 && c <= 0x36f
                || c >= 0x370 && c <= 0x37d
                || c >= 0x37f && c <= 0x1fff
                || c >= 0x200c && c <= 0x200d
                || c >= 0x203f && c <= 0x2040
                || c >= 0x2070 && c <= 0x218f
                || c >= 0x2c00 && c <= 0x2fef
                || c >= 0x3001 && c <= 0xd7ff
                || c >= 0xf900 && c <= 0xfdcf
                || c >= 0xfdf0 && c <= 0xfffd
                || c >= 0x10000 && c <= 0xeffff)
        {
            if count > XML_PARSER_CHUNK_SIZE {
                count = 0;
                GROW(ctxt);
            }
            count = count + 1;
            len += l;
            NEXTL(ctxt, l);
            unsafe { c = xmlCurrentChar(ctxt, &mut l) }
        }
    } else {
        if c == ' ' as i32
            || c == '>' as i32
            || c == '/' as i32
            || !IS_LETTER(c, unsafe { &xmlIsBaseCharGroup }) && c != '_' as i32 && c != ':' as i32
        {
            return 0 as *const xmlChar;
        }
        len += l;
        NEXTL(ctxt, l);
        unsafe {
            c = xmlCurrentChar(ctxt, &mut l);
        }
        while c != ' ' as i32
            && c != '>' as i32
            && c != '/' as i32
            && (IS_LETTER(c, unsafe { &xmlIsBaseCharGroup })
                || IS_DIGIT(c, unsafe { &xmlIsBaseCharGroup })
                || c == '.' as i32
                || c == '-' as i32
                || c == '_' as i32
                || c == ':' as i32
                || IS_COMBINING(c, unsafe { &xmlIsBaseCharGroup })
                || IS_EXTENDER(c, unsafe { &xmlIsBaseCharGroup }))
        {
            if count > XML_PARSER_CHUNK_SIZE {
                count = 0;
                GROW(ctxt);
                if unsafe { (*ctxt).instate == XML_PARSER_EOF } {
                    return 0 as *const xmlChar;
                }
            }
            count = count + 1;
            len += l;
            NEXTL(ctxt, l);
            unsafe { c = xmlCurrentChar(ctxt, &mut l) }
        }
    }
    if len > XML_MAX_NAME_LENGTH && unsafe { (*ctxt).options & XML_PARSE_HUGE as i32 == 0 } {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_NAME_TOO_LONG,
                b"Name\x00" as *const u8 as *const i8,
            );
        }
        return 0 as *const xmlChar;
    }
    if unsafe { ((*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64) < len as i64 } {
        /*
         * There were a couple of bugs where PERefs lead to to a change
         * of the buffer. Check the buffer size to avoid passing an invalid
         * pointer to xmlDictLookup.
         */
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"unexpected change of input buffer\x00" as *const u8 as *const i8,
            );
        }
        return 0 as *const xmlChar;
    }
    if unsafe {
        *(*(*ctxt).input).cur == '\n' as u8
            && *(*(*ctxt).input).cur.offset(-(1) as isize) == '\r' as u8
    } {
        return unsafe {
            xmlDictLookup_safe(
                (*ctxt).dict,
                (*(*ctxt).input).cur.offset(-((len + 1) as isize)),
                len,
            )
        };
    }
    return unsafe {
        xmlDictLookup_safe(
            (*ctxt).dict,
            (*(*ctxt).input).cur.offset(-(len as isize)),
            len,
        )
    };
}
/* *
* xmlParseName:
* @ctxt:  an XML parser context
*
* parse an XML name.
*
* [4] NameChar ::= Letter | Digit | '.' | '-' | '_' | ':' |
*                  CombiningChar | Extender
*
* [5] Name ::= (Letter | '_' | ':') (NameChar)*
*
* [6] Names ::= Name (#x20 Name)*
*
* Returns the Name parsed or NULL
*/

pub fn xmlParseName(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut in_0: *const xmlChar;
    let mut ret: *const xmlChar;
    let mut count: i32 = 0;
    GROW(ctxt);
    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseName = nbParseName + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    /*
     * Accelerator for simple ASCII names
     */
    in_0 = unsafe { (*(*ctxt).input).cur };
    if unsafe {
        *in_0 >= 0x61 && *in_0 <= 0x7a
            || *in_0 >= 0x41 && *in_0 <= 0x5a
            || *in_0 == '_' as u8
            || *in_0 == ':' as u8
    } {
        in_0 = unsafe { in_0.offset(1) };
        while unsafe {
            *in_0 >= 0x61 && *in_0 <= 0x7a
                || *in_0 >= 0x41 && *in_0 <= 0x5a
                || *in_0 >= 0x30 && *in_0 <= 0x39
                || *in_0 == '_' as u8
                || *in_0 == '-' as u8
                || *in_0 == ':' as u8
                || *in_0 == '.' as u8
        } {
            in_0 = unsafe { in_0.offset(1) };
        }
        if unsafe { *in_0 > 0 && *in_0 < 0x80 } {
            count = unsafe { in_0.offset_from((*(*ctxt).input).cur) as i32 };
            if count > XML_MAX_NAME_LENGTH
                && unsafe { (*ctxt).options & XML_PARSE_HUGE as i32 == 0 }
            {
                unsafe {
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_NAME_TOO_LONG,
                        b"Name\x00" as *const u8 as *const i8,
                    );
                }
                return 0 as *const xmlChar;
            }
            unsafe {
                ret = xmlDictLookup_safe((*ctxt).dict, (*(*ctxt).input).cur, count);
                (*(*ctxt).input).cur = in_0;
                (*(*ctxt).input).col += count;
            }
            if ret.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
            }
            return ret;
        }
    }
    /* accelerator for special cases */
    return xmlParseNameComplex(ctxt);
}
fn xmlParseNCNameComplex(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut len: i32 = 0;
    let mut l: i32 = 0;
    let mut c: i32 = 0;
    let mut count: i32 = 0;
    let mut startPosition: size_t = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseNCNameComplex = nbParseNCNameComplex + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    /*
     * Handler for more complex cases
     */
    GROW(ctxt);
    startPosition = unsafe { (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as size_t };
    c = unsafe { xmlCurrentChar(ctxt, &mut l) };
    if c == ' ' as i32
        || c == '>' as i32
        || c == '/' as i32
        || (xmlIsNameStartChar(ctxt, c) == 0 || c == ':' as i32)
    {
        return 0 as *const xmlChar;
    }
    while c != ' ' as i32
        && c != '>' as i32
        && c != '/' as i32
        && (xmlIsNameChar(ctxt, c) != 0 && c != ':' as i32)
    {
        if count > XML_PARSER_CHUNK_SIZE {
            if len > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                unsafe {
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_NAME_TOO_LONG,
                        b"NCName\x00" as *const u8 as *const i8,
                    );
                }
                return 0 as *const xmlChar;
            }
            count = 0;
            GROW(ctxt);
            if (safe_ctxt).instate == XML_PARSER_EOF {
                return 0 as *const xmlChar;
            }
        } else {
            count = count + 1;
        }
        len += l;
        NEXTL(ctxt, l);
        unsafe {
            c = xmlCurrentChar(ctxt, &mut l);
        }
        if c == 0 {
            count = 0;
            /*
             * when shrinking to extend the buffer we really need to preserve
             * the part of the name we already parsed. Hence rolling back
             * by current length.
             */
            unsafe { (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(-(l as isize)) };
            GROW(ctxt);
            if (safe_ctxt).instate == XML_PARSER_EOF {
                return 0 as *const xmlChar;
            }
            unsafe {
                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(l as isize);
                c = xmlCurrentChar(ctxt, &mut l)
            }
        }
    }
    if len > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_NAME_TOO_LONG,
                b"NCName\x00" as *const u8 as *const i8,
            );
        }
        return 0 as *const xmlChar;
    }
    return unsafe {
        xmlDictLookup_safe(
            (*ctxt).dict,
            (*(*ctxt).input).base.offset(startPosition as isize),
            len,
        )
    };
}
/* *
* xmlParseNCName:
* @ctxt:  an XML parser context
* @len:  length of the string parsed
*
* parse an XML name.
*
* [4NS] NCNameChar ::= Letter | Digit | '.' | '-' | '_' |
*                      CombiningChar | Extender
*
* [5NS] NCName ::= (Letter | '_') (NCNameChar)*
*
* Returns the Name parsed or NULL
*/
fn xmlParseNCName(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut in_0: *const xmlChar;
    let mut e: *const xmlChar;
    let mut ret: *const xmlChar;
    let mut count: i32 = 0;

    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseNCName = nbParseNCName + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    /*
     * Accelerator for simple ASCII names
     */
    unsafe {
        in_0 = (*(*ctxt).input).cur;
        e = (*(*ctxt).input).end;
    }
    if unsafe {
        (*in_0 >= 0x61 && *in_0 <= 0x7a || *in_0 >= 0x41 && *in_0 <= 0x5a || *in_0 == '_' as u8)
            && in_0 < e
    } {
        unsafe { in_0 = in_0.offset(1) };
        while unsafe {
            (*in_0 >= 0x61 && *in_0 <= 0x7a
                || *in_0 >= 0x41 && *in_0 <= 0x5a
                || *in_0 >= 0x30 && *in_0 <= 0x39
                || *in_0 == '_' as u8
                || *in_0 == '-' as u8
                || *in_0 == '.' as u8)
        } && in_0 < e
        {
            in_0 = unsafe { in_0.offset(1) };
        }
        if !(in_0 >= e) {
            if unsafe { *in_0 > 0 && *in_0 < 0x80 } {
                count = unsafe { in_0.offset_from((*(*ctxt).input).cur) as i32 };
                if count > XML_MAX_NAME_LENGTH
                    && unsafe { (*ctxt).options & XML_PARSE_HUGE as i32 == 0 }
                {
                    unsafe {
                        xmlFatalErr(
                            ctxt,
                            XML_ERR_NAME_TOO_LONG,
                            b"NCName\x00" as *const u8 as *const i8,
                        );
                    }
                    return 0 as *const xmlChar;
                }
                unsafe {
                    ret = xmlDictLookup_safe((*ctxt).dict, (*(*ctxt).input).cur, count);
                    (*(*ctxt).input).cur = in_0;
                    (*(*ctxt).input).col += count;
                }
                if ret.is_null() {
                    unsafe {
                        xmlErrMemory(ctxt, 0 as *const i8);
                    }
                }
                return ret;
            }
        }
    }
    return xmlParseNCNameComplex(ctxt);
}
/* *
* xmlParseNameAndCompare:
* @ctxt:  an XML parser context
*
* parse an XML name and compares for match
* (specialized for endtag parsing)
*
* Returns NULL for an illegal name, (xmlChar*) 1 for success
* and the name for mismatch
*/
fn xmlParseNameAndCompare(ctxt: xmlParserCtxtPtr, other: *const xmlChar) -> *const xmlChar {
    let mut cmp: *const xmlChar = other;
    let mut in_0: *const xmlChar;
    let ret: *const xmlChar;
    let safe_ctxt = unsafe { &mut *ctxt };
    GROW(safe_ctxt);
    if (safe_ctxt).instate == XML_PARSER_EOF {
        return 0 as *const xmlChar;
    }
    in_0 = unsafe { (*(*ctxt).input).cur };
    while unsafe { *in_0 != 0 && *in_0 == *cmp } {
        unsafe {
            in_0 = in_0.offset(1);
            cmp = cmp.offset(1);
        }
    }
    if unsafe {
        *cmp as i32 == 0
            && (*in_0 == '>' as u8
                || (*in_0 == 0x20 || 0x9 <= *in_0 && *in_0 <= 0xa || *in_0 == 0xd))
    } {
        /* success */
        unsafe {
            (*(*ctxt).input).col = ((*(*ctxt).input).col as i64
                + in_0.offset_from((*(*ctxt).input).cur) as i64)
                as i32;
            (*(*ctxt).input).cur = in_0;
        }
        return 1 as *const xmlChar;
    }
    /* failure (or end of input buffer), check with full function */
    ret = xmlParseName(ctxt);
    /* strings coming from the dictionary direct compare possible */
    if ret == other {
        return 1 as *const xmlChar;
    }
    return ret;
}
/* *
* xmlParseStringName:
* @ctxt:  an XML parser context
* @str:  a pointer to the string pointer (IN/OUT)
*
* parse an XML name.
*
* [4] NameChar ::= Letter | Digit | '.' | '-' | '_' | ':' |
*                  CombiningChar | Extender
*
* [5] Name ::= (Letter | '_' | ':') (NameChar)*
*
* [6] Names ::= Name (#x20 Name)*
*
* Returns the Name parsed or NULL. The @str pointer
* is updated to the current location in the string.
*/
fn xmlParseStringName(ctxt: xmlParserCtxtPtr, str: *mut *const xmlChar) -> *mut xmlChar {
    let mut buf: [xmlChar; XML_MAX_NAMELEN + 5] = [0; XML_MAX_NAMELEN + 5];
    let mut cur: *const xmlChar = unsafe { *str };
    let mut len: i32 = 0;
    let mut l: i32 = 0;
    let mut c: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseStringName = nbParseStringName + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    c = unsafe { xmlStringCurrentChar(ctxt, cur, &mut l) };
    if unsafe { xmlIsNameStartChar(ctxt, c) == 0 } {
        return 0 as *mut xmlChar;
    }
    if l == 1 {
        buf[len as usize] = c as xmlChar;
        len = len + 1;
    } else {
        len += unsafe { xmlCopyCharMultiByte(&mut *buf.as_mut_ptr().offset(len as isize), c) };
    }
    unsafe {
        cur = cur.offset(l as isize);
        c = xmlStringCurrentChar(ctxt, cur, &mut l);
    }
    while xmlIsNameChar(ctxt, c) != 0 {
        if l == 1 {
            buf[len as usize] = c as xmlChar;
            len = len + 1;
        } else {
            len += unsafe { xmlCopyCharMultiByte(&mut *buf.as_mut_ptr().offset(len as isize), c) };
        }
        unsafe {
            cur = cur.offset(l as isize);
            c = xmlStringCurrentChar(ctxt, cur, &mut l);
        }
        if len >= XML_MAX_NAMELEN as i32 {
            /* test bigentname.xml */
            /*
             * Okay someone managed to make a huge name, so he's ready to pay
             * for the processing speed.
             */
            let mut buffer: *mut xmlChar = 0 as *mut xmlChar;
            let mut max: i32 = len * 2;
            buffer = unsafe {
                xmlMallocAtomic_safe((max as u64).wrapping_mul(size_of::<xmlChar>() as u64))
            } as *mut xmlChar;
            if buffer.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                return 0 as *mut xmlChar;
            }
            unsafe { memcpy_safe(buffer as *mut (), buf.as_mut_ptr() as *const (), len as u64) };
            while xmlIsNameChar(ctxt, c) != 0 {
                if len + 10 > max {
                    let mut tmp: *mut xmlChar;
                    if len > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
                    {
                        unsafe {
                            xmlFatalErr(
                                ctxt,
                                XML_ERR_NAME_TOO_LONG,
                                b"NCName\x00" as *const u8 as *const i8,
                            );
                        }
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                    max *= 2;
                    tmp = unsafe {
                        xmlRealloc_safe(
                            buffer as *mut (),
                            (max as u64).wrapping_mul(size_of::<xmlChar>() as u64),
                        )
                    } as *mut xmlChar;
                    if tmp.is_null() {
                        unsafe {
                            xmlErrMemory(ctxt, 0 as *const i8);
                        }
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                    buffer = tmp
                }
                if l == 1 {
                    unsafe { *buffer.offset(len as isize) = c as xmlChar }
                    len = len + 1;
                } else {
                    len += unsafe { xmlCopyCharMultiByte(&mut *buffer.offset(len as isize), c) }
                }
                unsafe {
                    cur = cur.offset(l as isize);
                    c = xmlStringCurrentChar(ctxt, cur, &mut l);
                }
            }
            unsafe {
                *buffer.offset(len as isize) = 0;
                *str = cur;
            }
            return buffer;
        }
    }
    if len > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_NAME_TOO_LONG,
                b"NCName\x00" as *const u8 as *const i8,
            );
        }
        return 0 as *mut xmlChar;
    }
    unsafe {
        *str = cur;
    }
    return unsafe { xmlStrndup_safe(buf.as_mut_ptr(), len) };
}
/* *
* xmlParseNmtoken:
* @ctxt:  an XML parser context
*
* parse an XML Nmtoken.
*
* [7] Nmtoken ::= (NameChar)+
*
* [8] Nmtokens ::= Nmtoken (#x20 Nmtoken)*
*
* Returns the Nmtoken parsed or NULL
*/

pub fn xmlParseNmtoken(ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut buf: [xmlChar; XML_MAX_NAMELEN + 5] = [0; XML_MAX_NAMELEN + 5];
    let mut len: i32 = 0;
    let mut l: i32 = 0;
    let mut c: i32 = 0;
    let mut count: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };

    match () {
        #[cfg(HAVE_parser_DEBUG)]
        _ => {
            nbParseNmToken = nbParseNmToken + 1;
        }
        #[cfg(not(HAVE_parser_DEBUG))]
        _ => {}
    };

    GROW(ctxt);
    if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
        return 0 as *mut xmlChar;
    }
    unsafe {
        c = xmlCurrentChar(ctxt, &mut l);
    }
    while xmlIsNameChar(ctxt, c) != 0 {
        if count > 100 {
            count = 0;
            if (safe_ctxt).progressive == 0
                && unsafe { ((*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64) < 250 }
            {
                xmlGROW(ctxt);
            }
        } else {
            count += 1;
        }
        if l == 1 {
            buf[len as usize] = c as xmlChar;
            len = len + 1;
        } else {
            len += unsafe { xmlCopyCharMultiByte(&mut *buf.as_mut_ptr().offset(len as isize), c) };
        }
        NEXTL(ctxt, l);
        unsafe {
            c = xmlCurrentChar(ctxt, &mut l);
        }
        if c == 0 {
            count = 0;
            GROW(ctxt);
            if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
                return 0 as *mut xmlChar;
            }
            c = unsafe { xmlCurrentChar(ctxt, &mut l) };
        }
        if len >= XML_MAX_NAMELEN as i32 {
            /*
             * Okay someone managed to make a huge token, so he's ready to pay
             * for the processing speed.
             */
            let mut buffer: *mut xmlChar;
            let mut max: i32 = len * 2;
            buffer = unsafe {
                xmlMallocAtomic_safe((max as u64).wrapping_mul(size_of::<xmlChar>() as u64))
            } as *mut xmlChar;
            if buffer.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                return 0 as *mut xmlChar;
            }
            unsafe { memcpy_safe(buffer as *mut (), buf.as_mut_ptr() as *const (), len as u64) };
            while xmlIsNameChar(ctxt, c) != 0 {
                if count > XML_PARSER_CHUNK_SIZE {
                    count = 0;
                    GROW(ctxt);
                    if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                } else {
                    count += 1;
                }
                if len + 10 > max {
                    let mut tmp: *mut xmlChar;
                    if max > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
                    {
                        unsafe {
                            xmlFatalErr(
                                ctxt,
                                XML_ERR_NAME_TOO_LONG,
                                b"NmToken\x00" as *const u8 as *const i8,
                            );
                        }
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                    max *= 2;
                    tmp = unsafe {
                        xmlRealloc_safe(buffer as *mut (), max as u64 * size_of::<xmlChar>() as u64)
                    } as *mut xmlChar;
                    if tmp.is_null() {
                        unsafe {
                            xmlErrMemory(ctxt, 0 as *const i8);
                        }
                        unsafe { xmlFree_safe(buffer as *mut ()) };
                        return 0 as *mut xmlChar;
                    }
                    buffer = tmp
                }
                if l == 1 {
                    let fresh58 = len;
                    len = len + 1;
                    unsafe { *buffer.offset(fresh58 as isize) = c as xmlChar };
                } else {
                    len += unsafe { xmlCopyCharMultiByte(&mut *buffer.offset(len as isize), c) };
                }
                // COPY_BUF(l, &mut *buffer.as_mut_ptr(), len, c);
                NEXTL(ctxt, l);
                unsafe { c = xmlCurrentChar(ctxt, &mut l) }
            }
            unsafe {
                *buffer.offset(len as isize) = 0;
            }
            return buffer;
        }
    }
    if len == 0 {
        return 0 as *mut xmlChar;
    }
    if len > XML_MAX_NAME_LENGTH && unsafe { (*ctxt).options & XML_PARSE_HUGE as i32 == 0 } {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_NAME_TOO_LONG,
                b"NmToken\x00" as *const u8 as *const i8,
            );
        }
        return 0 as *mut xmlChar;
    }
    return unsafe { xmlStrndup_safe(buf.as_mut_ptr(), len) };
}
/* *
* xmlParseEntityValue:
* @ctxt:  an XML parser context
* @orig:  if non-NULL store a copy of the original entity value
*
* parse a value for ENTITY declarations
*
* [9] EntityValue ::= '"' ([^%&"] | PEReference | Reference)* '"' |
*	               "'" ([^%&'] | PEReference | Reference)* "'"
*
* Returns the EntityValue parsed with reference substituted or NULL
*/

pub fn xmlParseEntityValue(ctxt: xmlParserCtxtPtr, orig: *mut *mut xmlChar) -> *mut xmlChar {
    let mut current_block: u64;
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = XML_PARSER_BUFFER_SIZE as i32;
    let mut c: i32 = 0;
    let mut l: i32 = 0;
    let stop: xmlChar;
    let mut ret: *mut xmlChar = 0 as *mut xmlChar;
    let mut cur: *const xmlChar = 0 as *const xmlChar;
    let input: xmlParserInputPtr;
    let safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*(*ctxt).input).cur == '\"' as u8 } {
        stop = '\"' as xmlChar
    } else if unsafe { *(*(*ctxt).input).cur == '\'' as u8 } {
        stop = '\'' as xmlChar
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ENTITY_NOT_STARTED, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    buf = unsafe { xmlMallocAtomic_safe((size as u64).wrapping_mul(size_of::<xmlChar>() as u64)) }
        as *mut xmlChar;
    if buf.is_null() {
        unsafe {
            xmlErrMemory(ctxt, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    /*
     * The content of the entity definition is copied in a buffer.
     */
    (safe_ctxt).instate = XML_PARSER_ENTITY_VALUE;
    input = (safe_ctxt).input;
    GROW(ctxt);
    if !((safe_ctxt).instate == XML_PARSER_EOF) {
        unsafe { xmlNextChar_safe(ctxt) };
        unsafe { c = xmlCurrentChar(ctxt, &mut l) };
        loop
        /*
         * NOTE: 4.4.5 Included in Literal
         * When a parameter entity reference appears in a literal entity
         * value, ... a single or double quote character in the replacement
         * text is always treated as a normal data character and will not
         * terminate the literal.
         * In practice it means we stop the loop only when back at parsing
         * the initial entity and the quote is found
         */
        {
            if !(IS_CHAR(c)
                && (c != stop as i32 || (safe_ctxt).input != input)
                && (safe_ctxt).instate != XML_PARSER_EOF as i32)
            {
                current_block = 13460095289871124136;
                break;
            }
            if len + 5 >= size {
                let tmp: *mut xmlChar;
                size *= 2;
                tmp = unsafe {
                    xmlRealloc_safe(buf as *mut (), size as u64 * size_of::<xmlChar>() as u64)
                } as *mut xmlChar;
                if tmp.is_null() {
                    unsafe {
                        xmlErrMemory(ctxt, 0 as *const i8);
                    }
                    current_block = 1;
                    break;
                } else {
                    buf = tmp
                }
            }
            if l == 1 {
                let fresh59 = len;
                len = len + 1;
                unsafe { *buf.offset(fresh59 as isize) = c as xmlChar }
            } else {
                len += unsafe { xmlCopyCharMultiByte(&mut *buf.offset(len as isize), c) };
            }
            NEXTL(ctxt, l);
            GROW(ctxt);
            unsafe { c = xmlCurrentChar(ctxt, &mut l) };
            if c == 0 {
                GROW(ctxt);
                unsafe { c = xmlCurrentChar(ctxt, &mut l) };
            }
        }
        match current_block {
            1 => {}
            _ => {
                unsafe {
                    *buf.offset(len as isize) = 0;
                }
                if !((safe_ctxt).instate as i32 == XML_PARSER_EOF as i32) {
                    if c != stop as i32 {
                        unsafe {
                            xmlFatalErr(ctxt, XML_ERR_ENTITY_NOT_FINISHED, 0 as *const i8);
                        }
                    } else {
                        unsafe { xmlNextChar_safe(ctxt) };
                        /*
                         * Raise problem w.r.t. '&' and '%' being used in non-entities
                         * reference constructs. Note Charref will be handled in
                         * xmlStringDecodeEntities()
                         */
                        cur = buf;
                        loop {
                            if !(unsafe { *cur } != 0) {
                                current_block = 2;
                                break;
                            }
                            /* non input consuming */
                            if unsafe {
                                *cur == '%' as u8
                                    || *cur == '&' as u8 && *cur.offset(1) != '#' as u8
                            } {
                                let name: *mut xmlChar;
                                let tmp_0: xmlChar = unsafe { *cur };
                                let mut nameOk: i32 = 0;
                                unsafe {
                                    cur = cur.offset(1);
                                }
                                name = xmlParseStringName(ctxt, &mut cur);
                                if !name.is_null() {
                                    nameOk = 1;
                                    unsafe { xmlFree_safe(name as *mut ()) };
                                }
                                if nameOk == 0 || unsafe { *cur != ';' as u8 } {
                                    unsafe {
                                        xmlFatalErrMsgInt(ctxt,
                                                                  XML_ERR_ENTITY_CHAR_ERROR,
                                                                  b"EntityValue: \'%c\' forbidden except for entities references\n\x00"
                                                                  as *const u8 as
                                                                  *const i8,
                                                                  tmp_0 as i32);
                                    }
                                    current_block = 1;
                                    break;
                                } else if tmp_0 == '%' as u8
                                    && (safe_ctxt).inSubset == 1
                                    && (safe_ctxt).inputNr == 1
                                {
                                    unsafe {
                                        xmlFatalErr(
                                            ctxt,
                                            XML_ERR_ENTITY_PE_INTERNAL,
                                            0 as *const i8,
                                        );
                                    }
                                    current_block = 1;
                                    break;
                                } else if unsafe { *cur == 0 } {
                                    current_block = 2;
                                    break;
                                }
                            }
                            cur = unsafe { cur.offset(1) };
                        }
                        match current_block {
                            1 => {}
                            _ => {
                                /*
                                 * Then PEReference entities are substituted.
                                 *
                                 * NOTE: 4.4.7 Bypassed
                                 * When a general entity reference appears in the EntityValue in
                                 * an entity declaration, it is bypassed and left as is.
                                 * so XML_SUBSTITUTE_REF is not set here.
                                 */
                                (safe_ctxt).depth += 1;
                                ret = xmlStringDecodeEntities(ctxt, buf, 2, 0, 0, 0);
                                (safe_ctxt).depth -= 1;
                                if !orig.is_null() {
                                    unsafe {
                                        *orig = buf;
                                    }
                                    buf = 0 as *mut xmlChar
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    if !buf.is_null() {
        unsafe { xmlFree_safe(buf as *mut ()) };
    }
    return ret;
}
/* *
* xmlParseAttValueComplex:
* @ctxt:  an XML parser context
* @len:   the resulting attribute len
* @normalize:  whether to apply the inner normalization
*
* parse a value for an attribute, this is the fallback function
* of xmlParseAttValue() when the attribute parsing requires handling
* of non-ASCII characters, or normalization compaction.
*
* Returns the AttValue parsed or NULL. The value has to be freed by the caller.
*/

fn growBuffer(mut buf: *mut xmlChar, n: u64, mut buf_size: size_t, mut current_block: u64) -> bool {
    let mut tmp: *mut xmlChar = 0 as *mut xmlChar;
    let new_size: size_t = buf_size.wrapping_mul(2).wrapping_add(n);
    if new_size < buf_size {
        current_block = 1;
        return true;
    }
    tmp = unsafe { xmlRealloc_safe(buf as *mut (), new_size) } as *mut xmlChar;
    if tmp.is_null() {
        current_block = 1;
        return true;
    }
    buf = tmp;
    buf_size = new_size;
    return false;
}
const XML_MAX_TEXT_LENGTH: u64 = 10000000;
fn xmlParseAttValueComplex(
    ctxt: xmlParserCtxtPtr,
    attlen: *mut i32,
    normalize: i32,
) -> *mut xmlChar {
    let mut current_block: u64 = 0;
    let mut limit: xmlChar = 0;
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut rep: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: size_t = 0;
    let mut buf_size: size_t = 0;
    let mut c: i32 = 0;
    let mut l: i32 = 0;
    let mut in_space: i32 = 0;
    let mut current: *mut xmlChar = 0 as *mut xmlChar;
    let mut ent: xmlEntityPtr;
    let safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*(*ctxt).input).cur.offset(0) == '\"' as u8 } {
        (safe_ctxt).instate = XML_PARSER_ATTRIBUTE_VALUE;
        limit = '\"' as xmlChar;
        unsafe { xmlNextChar_safe(ctxt) };
    } else if unsafe { *(*(*ctxt).input).cur.offset(0) == '\'' as u8 } {
        limit = '\'' as xmlChar;
        (safe_ctxt).instate = XML_PARSER_ATTRIBUTE_VALUE;
        unsafe { xmlNextChar_safe(ctxt) };
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ATTRIBUTE_NOT_STARTED, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    /*
     * allocate a translation buffer.
     */
    buf_size = XML_PARSER_BUFFER_SIZE;
    buf = unsafe { xmlMallocAtomic_safe(buf_size) } as *mut xmlChar;
    if buf.is_null() {
        current_block = 1;
    } else {
        /*
         * OK loop until we reach one of the ending char or a size limit.
         */
        unsafe {
            c = xmlCurrentChar(ctxt, &mut l);
        }
        loop {
            if unsafe {
                !(*(*(*ctxt).input).cur.offset(0) != limit
                    && IS_CHAR(c)
                    && c != '<' as i32
                    && (safe_ctxt).instate != XML_PARSER_EOF as i32)
            } {
                current_block = 3166194604430448652;
                break;
            }
            /*
             * Impose a reasonable limit on attribute size, unless XML_PARSE_HUGE
             * special option is given
             */
            if len > XML_MAX_TEXT_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                unsafe {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ATTRIBUTE_NOT_FINISHED,
                        b"AttValue length too long\n\x00" as *const u8 as *const i8,
                    );
                }
                current_block = 1;
                break;
            } else {
                if c == '&' as i32 {
                    in_space = 0;
                    if unsafe { *(*(*ctxt).input).cur.offset(1) == '#' as u8 } {
                        let val: i32 = xmlParseCharRef(ctxt);
                        if val == '&' as i32 {
                            if (safe_ctxt).replaceEntities != 0 {
                                if len + 10 > buf_size {
                                    // if(growBuffer(buf, 10, buf_size, current_block)) {
                                    //     break;
                                    // }
                                    let mut tmp_1: *mut xmlChar = 0 as *mut xmlChar;
                                    let new_size_1: size_t =
                                        buf_size.wrapping_mul(2).wrapping_add(10);
                                    if new_size_1 < buf_size {
                                        current_block = 1;
                                        break;
                                    }
                                    tmp_1 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_1) }
                                        as *mut xmlChar;
                                    if tmp_1.is_null() {
                                        current_block = 1;
                                        break;
                                    }
                                    buf = tmp_1;
                                    buf_size = new_size_1
                                }
                                unsafe { *buf.offset(len as isize) = '&' as u8 };
                                len += 1;
                            } else {
                                /*
                                 * The reparsing will be done in xmlStringGetNodeList()
                                 * called by the attribute() function in SAX.c
                                 */
                                if len + 10 > buf_size {
                                    // if(growBuffer(buf, 10, buf_size, current_block)){
                                    //     break;
                                    // }
                                    let mut tmp_1: *mut xmlChar = 0 as *mut xmlChar;
                                    let new_size_1: size_t =
                                        buf_size.wrapping_mul(2).wrapping_add(10);
                                    if new_size_1 < buf_size {
                                        current_block = 1;
                                        break;
                                    }
                                    tmp_1 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_1) }
                                        as *mut xmlChar;
                                    if tmp_1.is_null() {
                                        current_block = 1;
                                        break;
                                    }
                                    buf = tmp_1;
                                    buf_size = new_size_1
                                }
                                unsafe {
                                    *buf.offset(len as isize) = '&' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '#' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '3' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '8' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = ';' as xmlChar;
                                    len += 1;
                                }
                            }
                        } else if val != 0 {
                            if len + 10 > buf_size {
                                // if(growBuffer(buf, 10, buf_size, current_block)){
                                //     break;
                                // }

                                let mut tmp_1: *mut xmlChar = 0 as *mut xmlChar;
                                let new_size_1: size_t = buf_size.wrapping_mul(2).wrapping_add(10);
                                if new_size_1 < buf_size {
                                    current_block = 1;
                                    break;
                                }
                                tmp_1 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_1) }
                                    as *mut xmlChar;
                                if tmp_1.is_null() {
                                    current_block = 1;
                                    break;
                                }
                                buf = tmp_1;
                                buf_size = new_size_1
                            }
                            len += unsafe {
                                xmlCopyChar(0, &mut *buf.offset(len as isize), val) as size_t
                            };
                        }
                    } else {
                        ent = unsafe { xmlParseEntityRef(ctxt) };
                        (safe_ctxt).nbentities += 1;
                        if !ent.is_null() {
                            unsafe {
                                (*ctxt).nbentities += (*ent).owner as u64;
                            };
                        }
                        if !ent.is_null()
                            && unsafe {
                                (*ent).etype as u32 == XML_INTERNAL_PREDEFINED_ENTITY as u32
                            }
                        {
                            if len + 10 > buf_size {
                                // if(growBuffer(buf, 10, buf_size, current_block)){
                                //     break;
                                // }
                                let mut tmp_1: *mut xmlChar = 0 as *mut xmlChar;
                                let new_size_1: size_t = buf_size.wrapping_mul(2).wrapping_add(10);
                                if new_size_1 < buf_size {
                                    current_block = 1;
                                    break;
                                }
                                tmp_1 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_1) }
                                    as *mut xmlChar;
                                if tmp_1.is_null() {
                                    current_block = 1;
                                    break;
                                }
                                buf = tmp_1;
                                buf_size = new_size_1
                            }
                            if unsafe {
                                (*ctxt).replaceEntities == 0
                                    && *(*ent).content.offset(0) == '&' as u8
                            } {
                                unsafe {
                                    *buf.offset(len as isize) = '&' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '#' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '3' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = '8' as xmlChar;
                                    len += 1;
                                    *buf.offset(len as isize) = ';' as xmlChar;
                                    len += 1;
                                }
                            } else {
                                unsafe { *buf.offset(len as isize) = *(*ent).content.offset(0) }
                                len += 1;
                            }
                        } else if !ent.is_null() && (safe_ctxt).replaceEntities != 0 {
                            if unsafe {
                                (*ent).etype as u32 != XML_INTERNAL_PREDEFINED_ENTITY as u32
                            } {
                                (safe_ctxt).depth += 1;
                                rep = unsafe {
                                    xmlStringDecodeEntities(ctxt, (*ent).content, 1, 0, 0, 0)
                                };
                                (safe_ctxt).depth -= 1;
                                if !rep.is_null() {
                                    current = rep;
                                    while unsafe { *current } != 0 {
                                        /* non input consuming */
                                        if unsafe { *current } == 0xd
                                            || unsafe { *current } == 0xa
                                            || unsafe { *current } == 0x9
                                        {
                                            unsafe {
                                                *buf.offset(len as isize) = 0x20;
                                                len += 1;
                                                current = current.offset(1);
                                            }
                                        } else {
                                            unsafe {
                                                *buf.offset(len as isize) = *current;
                                                current = current.offset(1);
                                                len += 1;
                                            }
                                        }
                                        if !(len + 10 > buf_size) {
                                            continue;
                                        }
                                        let tmp_3: *mut xmlChar;
                                        let new_size_3: size_t = buf_size * 2 + 10;
                                        if new_size_3 < buf_size {
                                            current_block = 3;
                                            break;
                                        }
                                        tmp_3 =
                                            unsafe { xmlRealloc_safe(buf as *mut (), new_size_3) }
                                                as *mut xmlChar;
                                        if tmp_3.is_null() {
                                            current_block = 3;
                                            break;
                                        }
                                        buf = tmp_3;
                                        buf_size = new_size_3
                                    }
                                    unsafe { xmlFree_safe(rep as *mut ()) };
                                    rep = 0 as *mut xmlChar
                                }
                            } else {
                                if len + 10 > buf_size {
                                    // if(growBuffer(buf, 10, buf_size, current_block)){
                                    //     break;
                                    // }
                                    let mut tmp_4: *mut xmlChar = 0 as *mut xmlChar;
                                    let new_size_4: size_t =
                                        buf_size.wrapping_mul(2).wrapping_add(10);
                                    if new_size_4 < buf_size {
                                        current_block = 3;
                                        break;
                                    }
                                    tmp_4 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_4) }
                                        as *mut xmlChar;
                                    if tmp_4.is_null() {
                                        current_block = 3;
                                        break;
                                    }
                                    buf = tmp_4;
                                    buf_size = new_size_4
                                }
                                if unsafe { !(*ent).content.is_null() } {
                                    unsafe {
                                        *buf.offset(len as isize) = *(*ent).content.offset(0)
                                    };
                                    len += 1;
                                }
                            }
                        } else if !ent.is_null() {
                            let safe_ent = unsafe { &mut *ent };
                            let mut i: i32 = unsafe { xmlStrlen_safe((safe_ent).name) };
                            let mut cur: *const xmlChar = (safe_ent).name;
                            /*
                             * This may look absurd but is needed to detect
                             * entities problems
                             */
                            if (safe_ent).etype != XML_INTERNAL_PREDEFINED_ENTITY as u32
                                && !(safe_ent).content.is_null()
                                && (safe_ent).checked == 0
                            {
                                let oldnbent: u64 = (safe_ctxt).nbentities;
                                let mut diff: u64 = 0;
                                (safe_ctxt).depth += 1;
                                rep = unsafe {
                                    xmlStringDecodeEntities(ctxt, (safe_ent).content, 1, 0, 0, 0)
                                };
                                (safe_ctxt).depth -= 1;
                                diff = (safe_ctxt).nbentities - oldnbent + 1;
                                if diff > (INT_MAX / 2) as u64 {
                                    diff = (INT_MAX / 2) as u64
                                }
                                (safe_ent).checked = (diff * 2) as i32;
                                if !rep.is_null() {
                                    if !unsafe { xmlStrchr_safe(rep, '<' as xmlChar) }.is_null() {
                                        (safe_ent).checked |= 1
                                    }
                                    unsafe { xmlFree_safe(rep as *mut ()) };
                                    rep = 0 as *mut xmlChar
                                } else {
                                    unsafe { *(*ent).content.offset(0) = 0 };
                                }
                            }
                            /*
                             * Just output the reference
                             */
                            unsafe {
                                *buf.offset(len as isize) = '&' as xmlChar;
                            }
                            len += 1;
                            while len + i as u64 + 10 > buf_size {
                                let tmp_5: *mut xmlChar;
                                let new_size_5: size_t = buf_size * 2 + i as u64 + 10;
                                if new_size_5 < buf_size {
                                    current_block = 1;
                                    break;
                                }
                                tmp_5 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_5) }
                                    as *mut xmlChar;
                                if tmp_5.is_null() {
                                    current_block = 1;
                                    break;
                                }
                                buf = tmp_5;
                                buf_size = new_size_5
                            }
                            while i > 0 {
                                unsafe {
                                    *buf.offset(len as isize) = *cur;
                                    cur = cur.offset(1);
                                    len += 1;
                                }
                                i -= 1
                            }
                            unsafe { *buf.offset(len as isize) = ';' as xmlChar };
                            len += 1;
                        }
                    }
                } else {
                    if c == 0x20 || c == 0xd || c == 0xa || c == 0x9 {
                        if len != 0 || normalize == 0 {
                            if normalize == 0 || in_space == 0 {
                                if l == 1 {
                                    let fresh80 = len;
                                    len += 1;
                                    unsafe { *buf.offset(fresh80 as isize) = 0x20 };
                                } else {
                                    len += unsafe {
                                        xmlCopyCharMultiByte(&mut *buf.offset(len as isize), 0x20)
                                            as size_t
                                    };
                                }
                                while len + 10 > buf_size {
                                    // if(growBuffer(buf, 10, buf_size, current_block)){
                                    //     break;
                                    // }
                                    let mut tmp_6: *mut xmlChar = 0 as *mut xmlChar;
                                    let new_size_6: size_t = buf_size * 2 + 10;
                                    if new_size_6 < buf_size {
                                        current_block = 1;
                                        break;
                                    }
                                    tmp_6 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_6) }
                                        as *mut xmlChar;
                                    if tmp_6.is_null() {
                                        current_block = 1;
                                        break;
                                    }
                                    buf = tmp_6;
                                    buf_size = new_size_6
                                }
                            }
                            in_space = 1
                        }
                    } else {
                        in_space = 0;
                        if l == 1 {
                            let fresh81 = len;
                            len += 1;
                            unsafe { *buf.offset(fresh81 as isize) = c as xmlChar };
                        } else {
                            len += unsafe {
                                xmlCopyCharMultiByte(&mut *buf.offset(len as isize), c) as size_t
                            };
                        }
                        if len + 10 > buf_size {
                            // if(growBuffer(buf, 10, buf_size, current_block)){
                            //     break;
                            // }
                            let mut tmp_7: *mut xmlChar = 0 as *mut xmlChar;
                            let new_size_7: size_t = buf_size.wrapping_mul(2).wrapping_add(10);
                            if new_size_7 < buf_size {
                                current_block = 1;
                                break;
                            }
                            tmp_7 = unsafe { xmlRealloc_safe(buf as *mut (), new_size_7) }
                                as *mut xmlChar;
                            if tmp_7.is_null() {
                                current_block = 1;
                                break;
                            }
                            buf = tmp_7;
                            buf_size = new_size_7
                        }
                    }
                    NEXTL(ctxt, l);
                }
                GROW(ctxt);
                unsafe { c = xmlCurrentChar(ctxt, &mut l) };
            }
        }
        match current_block {
            1 => {}
            _ => {
                if (safe_ctxt).instate == XML_PARSER_EOF {
                    current_block = 2;
                } else {
                    if in_space != 0 && normalize != 0 {
                        while len > 0 && unsafe { *buf.offset((len - 1) as isize) as i32 == 0x20 } {
                            len -= 1
                        }
                    }
                    unsafe {
                        *buf.offset(len as isize) = 0;
                    }
                    if unsafe { *(*(*ctxt).input).cur == '<' as u8 } {
                        unsafe {
                            xmlFatalErr(ctxt, XML_ERR_LT_IN_ATTRIBUTE, 0 as *const i8);
                        }
                    } else if unsafe { *(*(*ctxt).input).cur != limit as u8 } {
                        if c != 0 && !IS_CHAR(c) {
                            unsafe {
                                xmlFatalErrMsg(
                                    ctxt,
                                    XML_ERR_INVALID_CHAR,
                                    b"invalid character in attribute value\n\x00" as *const u8
                                        as *const i8,
                                );
                            }
                        } else {
                            unsafe {
                                xmlFatalErrMsg(
                                    ctxt,
                                    XML_ERR_ATTRIBUTE_NOT_FINISHED,
                                    b"AttValue: \' expected\n\x00" as *const u8 as *const i8,
                                );
                            }
                        }
                    } else {
                        unsafe { xmlNextChar_safe(ctxt) };
                    }
                    /*
                     * There we potentially risk an overflow, don't allow attribute value of
                     * length more than INT_MAX it is a very reasonable assumption !
                     */
                    if len >= INT_MAX as u64 {
                        unsafe {
                            xmlFatalErrMsg(
                                ctxt,
                                XML_ERR_ATTRIBUTE_NOT_FINISHED,
                                b"AttValue length too long\n\x00" as *const u8 as *const i8,
                            );
                        }
                    } else {
                        if !attlen.is_null() {
                            unsafe { *attlen = len as i32 };
                        }
                        return buf;
                    }
                    current_block = 1;
                }
            }
        }
    }
    match current_block {
        1 => unsafe {
            xmlErrMemory(ctxt, 0 as *const i8);
        },
        _ => {}
    }
    if !buf.is_null() {
        unsafe { xmlFree_safe(buf as *mut ()) };
    }
    if !rep.is_null() {
        unsafe { xmlFree_safe(rep as *mut ()) };
    }
    return 0 as *mut xmlChar;
}
/* *
* xmlParseAttValue:
* @ctxt:  an XML parser context
*
* parse a value for an attribute
* Note: the parser won't do substitution of entities here, this
* will be handled later in xmlStringGetNodeList
*
* [10] AttValue ::= '"' ([^<&"] | Reference)* '"' |
*                   "'" ([^<&'] | Reference)* "'"
*
* 3.3.3 Attribute-Value Normalization:
* Before the value of an attribute is passed to the application or
* checked for validity, the XML processor must normalize it as follows:
* - a character reference is processed by appending the referenced
*   character to the attribute value
* - an entity reference is processed by recursively processing the
*   replacement text of the entity
* - a whitespace character (#x20, #xD, #xA, #x9) is processed by
*   appending #x20 to the normalized value, except that only a single
*   #x20 is appended for a "#xD#xA" sequence that is part of an external
*   parsed entity or the literal entity value of an internal parsed entity
* - other characters are processed by appending them to the normalized value
* If the declared value is not CDATA, then the XML processor must further
* process the normalized attribute value by discarding any leading and
* trailing space (#x20) characters, and by replacing sequences of space
* (#x20) characters by a single space (#x20) character.
* All attributes for which no declaration has been read should be treated
* by a non-validating parser as if declared CDATA.
*
* Returns the AttValue parsed or NULL. The value has to be freed by the caller.
*/

pub fn xmlParseAttValue(ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    if ctxt.is_null() || unsafe { (*ctxt).input.is_null() } {
        return 0 as *mut xmlChar;
    }
    return unsafe { xmlParseAttValueInternal(ctxt, 0 as *mut i32, 0 as *mut i32, 0) };
}
/* *
* xmlParseSystemLiteral:
* @ctxt:  an XML parser context
*
* parse an XML Literal
*
* [11] SystemLiteral ::= ('"' [^"]* '"') | ("'" [^']* "'")
*
* Returns the SystemLiteral parsed or NULL
*/

pub fn xmlParseSystemLiteral(ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = XML_PARSER_BUFFER_SIZE as i32;
    let mut cur: i32;
    let mut l: i32 = 0;
    let mut stop: xmlChar;
    let safe_ctxt = unsafe { &mut *ctxt };
    let state: i32 = (safe_ctxt).instate as i32;
    let mut count: i32 = 0;
    SHRINK(ctxt);
    if unsafe { *(*(*ctxt).input).cur == '\"' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        stop = '\"' as xmlChar
    } else if unsafe { *(*(*ctxt).input).cur == '\'' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        stop = '\'' as xmlChar
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_LITERAL_NOT_STARTED, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    buf = unsafe { xmlMallocAtomic_safe((size as u64).wrapping_mul(size_of::<xmlChar>() as u64)) }
        as *mut xmlChar;
    if buf.is_null() {
        unsafe {
            xmlErrMemory(ctxt, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    (safe_ctxt).instate = XML_PARSER_SYSTEM_LITERAL;
    cur = unsafe { xmlCurrentChar(ctxt, &mut l) };
    while IS_CHAR(cur) && cur != stop as i32 {
        /* checked */
        if len + 5 >= size {
            let mut tmp: *mut xmlChar;
            if size > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                unsafe {
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_NAME_TOO_LONG,
                        b"SystemLiteral\x00" as *const u8 as *const i8,
                    );
                }
                unsafe { xmlFree_safe(buf as *mut ()) };
                (safe_ctxt).instate = state as xmlParserInputState;
                return 0 as *mut xmlChar;
            }
            size *= 2;
            tmp = unsafe {
                xmlRealloc_safe(buf as *mut (), size as u64 * size_of::<xmlChar>() as u64)
            } as *mut xmlChar;
            if tmp.is_null() {
                unsafe { xmlFree_safe(buf as *mut ()) };
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                (safe_ctxt).instate = state as xmlParserInputState;
                return 0 as *mut xmlChar;
            }
            buf = tmp
        }
        count += 1;
        if count > 50 {
            SHRINK(ctxt);
            GROW(ctxt);
            count = 0;
            if (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32 {
                unsafe { xmlFree_safe(buf as *mut ()) };
                return 0 as *mut xmlChar;
            }
        }
        if l == 1 {
            let fresh82 = len;
            len = len + 1;
            unsafe { *buf.offset(fresh82 as isize) = cur as xmlChar }
        } else {
            len += unsafe { xmlCopyCharMultiByte(&mut *buf.offset(len as isize), cur) };
        }
        NEXTL(ctxt, l);
        unsafe {
            cur = xmlCurrentChar(ctxt, &mut l);
        }
        if cur == 0 {
            GROW(ctxt);
            SHRINK(ctxt);
            cur = unsafe { xmlCurrentChar(ctxt, &mut l) };
        }
    }
    unsafe {
        *buf.offset(len as isize) = 0;
        (*ctxt).instate = state as xmlParserInputState;
    }
    if !IS_CHAR(cur) {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_LITERAL_NOT_FINISHED, 0 as *const i8);
        }
    } else {
        unsafe { xmlNextChar_safe(ctxt) };
    }
    return buf;
}
/* *
* xmlParsePubidLiteral:
* @ctxt:  an XML parser context
*
* parse an XML public literal
*
* [12] PubidLiteral ::= '"' PubidChar* '"' | "'" (PubidChar - "'")* "'"
*
* Returns the PubidLiteral parsed or NULL.
*/
pub fn xmlParsePubidLiteral(ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = XML_PARSER_BUFFER_SIZE as i32;
    let mut cur: xmlChar;
    let mut stop: xmlChar;
    let mut count: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    let oldstate: xmlParserInputState = (safe_ctxt).instate;
    SHRINK(ctxt);
    if unsafe { *(*(*ctxt).input).cur == '\"' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        stop = '\"' as xmlChar
    } else if unsafe { *(*(*ctxt).input).cur == '\'' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        stop = '\'' as xmlChar
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_LITERAL_NOT_STARTED, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    buf = unsafe { xmlMallocAtomic_safe((size as u64).wrapping_mul(size_of::<xmlChar>() as u64)) }
        as *mut xmlChar;
    if buf.is_null() {
        unsafe {
            xmlErrMemory(ctxt, 0 as *const i8);
        }
        return 0 as *mut xmlChar;
    }
    (safe_ctxt).instate = XML_PARSER_PUBLIC_LITERAL;
    cur = unsafe { *(*(*ctxt).input).cur };
    while unsafe { xmlIsPubidChar_tab[cur as usize] != 0 } && cur != stop {
        /* checked */
        if len + 1 >= size {
            let mut tmp: *mut xmlChar;
            if size > XML_MAX_NAME_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                unsafe {
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_NAME_TOO_LONG,
                        b"Public ID\x00" as *const u8 as *const i8,
                    );
                }
                unsafe { xmlFree_safe(buf as *mut ()) };
                return 0 as *mut xmlChar;
            }
            size *= 2;
            tmp = unsafe {
                xmlRealloc_safe(
                    buf as *mut (),
                    (size as u64).wrapping_mul(size_of::<xmlChar>() as u64),
                )
            } as *mut xmlChar;
            if tmp.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                unsafe { xmlFree_safe(buf as *mut ()) };
                return 0 as *mut xmlChar;
            }
            buf = tmp
        }
        unsafe {
            *buf.offset(len as isize) = cur;
        }
        len = len + 1;
        count += 1;
        if count > 50 {
            SHRINK(ctxt);
            GROW(ctxt);
            count = 0;
            if (safe_ctxt).instate == XML_PARSER_EOF {
                unsafe { xmlFree_safe(buf as *mut ()) };
                return 0 as *mut xmlChar;
            }
        }
        unsafe { xmlNextChar_safe(ctxt) };
        cur = unsafe { *(*(*ctxt).input).cur };
        if cur == 0 {
            SHRINK(ctxt);
            GROW(ctxt);
            cur = unsafe { *(*(*ctxt).input).cur };
        }
    }
    unsafe {
        *buf.offset(len as isize) = 0;
    }
    if cur != stop {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_LITERAL_NOT_FINISHED, 0 as *const i8);
        }
    } else {
        unsafe { xmlNextChar_safe(ctxt) };
    }
    (safe_ctxt).instate = oldstate;
    return buf;
}
/*
* used for the test in the inner loop of the char data testing
*/
static mut test_char_data: [u8; 256] = [
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, /* 0x9, CR/LF separated */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x20, 0x21, 0x22, 0x23, 0x24, 0x25, 0x00, 0x27, /* & */
    0x28, 0x29, 0x2a, 0x2b, 0x2c, 0x2d, 0x2e, 0x2f, 0x30, 0x31, 0x32, 0x33, 0x34, 0x35, 0x36, 0x37,
    0x38, 0x39, 0x3a, 0x3b, 0x00, 0x3d, 0x3e, 0x3f, /* < */
    0x40, 0x41, 0x42, 0x43, 0x44, 0x45, 0x46, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0x4c, 0x4d, 0x4e, 0x4f,
    0x50, 0x51, 0x52, 0x53, 0x54, 0x55, 0x56, 0x57, 0x58, 0x59, 0x5a, 0x5b, 0x5c, 0x00, 0x5e,
    0x5f, /* ] */
    0x60, 0x61, 0x62, 0x63, 0x64, 0x65, 0x66, 0x67, 0x68, 0x69, 0x6a, 0x6b, 0x6c, 0x6d, 0x6e, 0x6f,
    0x70, 0x71, 0x72, 0x73, 0x74, 0x75, 0x76, 0x77, 0x78, 0x79, 0x7a, 0x7b, 0x7c, 0x7d, 0x7e, 0x7f,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, /* non-ascii */
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
/* *
* xmlParseCharData:
* @ctxt:  an XML parser context
* @cdata:  int indicating whether we are within a CDATA section
*
* parse a CharData section.
* if we are within a CDATA section ']]>' marks an end of section.
*
* The right angle bracket (>) may be represented using the string "&gt;",
* and must, for compatibility, be escaped using "&gt;" or a character
* reference when it appears in the string "]]>" in content, when that
* string is not marking the end of a CDATA section.
*
* [14] CharData ::= [^<&]* - ([^<&]* ']]>' [^<&]*)
*/

pub fn xmlParseCharData(ctxt: xmlParserCtxtPtr, cdata: i32) {
    let safe_ctxt = unsafe { &mut *ctxt };
    let mut current_block: u64;
    let mut in_0: *const xmlChar = 0 as *const xmlChar;
    let mut nbchar: i32 = 0;
    let mut line: i32 = unsafe { (*(*ctxt).input).line };
    let mut col: i32 = unsafe { (*(*ctxt).input).col };
    let mut ccol: i32 = 0;
    SHRINK(ctxt);
    GROW(ctxt);
    /*
     * Accelerated common case where input don't need to be
     * modified before passing it to the handler.
     */
    if cdata == 0 {
        in_0 = unsafe { (*(*ctxt).input).cur };
        loop {
            while unsafe { *in_0 == 0x20 } {
                unsafe {
                    in_0 = in_0.offset(1);
                    (*(*ctxt).input).col += 1;
                }
            }
            if unsafe { *in_0 == 0xa } {
                loop {
                    unsafe {
                        (*(*ctxt).input).line += 1;
                        (*(*ctxt).input).col = 1;
                        in_0 = in_0.offset(1);
                    }
                    if unsafe { !(*in_0 == 0xa) } {
                        break;
                    }
                }
            } else {
                if unsafe { *in_0 == '<' as u8 } {
                    nbchar = unsafe { in_0.offset_from((*(*ctxt).input).cur) as i32 };
                    if nbchar > 0 {
                        let tmp: *const xmlChar = unsafe { (*(*ctxt).input).cur };
                        unsafe { (*(*ctxt).input).cur = in_0 };
                        if unsafe {
                            !(*ctxt).sax.is_null()
                                && (*(*ctxt).sax).ignorableWhitespace != (*(*ctxt).sax).characters
                        } {
                            if areBlanks(ctxt, tmp, nbchar, 1) != 0 {
                                if unsafe { (*(*ctxt).sax).ignorableWhitespace.is_some() } {
                                    unsafe {
                                        (*(*ctxt).sax)
                                            .ignorableWhitespace
                                            .expect("non-null function pointer")(
                                            (*ctxt).userData,
                                            tmp,
                                            nbchar,
                                        );
                                    }
                                }
                            } else {
                                if unsafe { (*(*ctxt).sax).characters.is_some() } {
                                    unsafe {
                                        (*(*ctxt).sax)
                                            .characters
                                            .expect("non-null function pointer")(
                                            (*ctxt).userData,
                                            tmp,
                                            nbchar,
                                        );
                                    }
                                }
                                if unsafe { *(*ctxt).space } == -1 {
                                    unsafe { *(*ctxt).space = -2 }
                                }
                            }
                        } else if !(safe_ctxt).sax.is_null()
                            && unsafe { (*(*ctxt).sax).characters.is_some() }
                        {
                            unsafe {
                                (*(*ctxt).sax)
                                    .characters
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    tmp,
                                    nbchar,
                                );
                            }
                        }
                    }
                    return;
                }
                loop {
                    unsafe {
                        ccol = (*(*ctxt).input).col;
                        while test_char_data[*in_0 as usize] != 0 {
                            in_0 = in_0.offset(1);
                            ccol += 1
                        }
                        (*(*ctxt).input).col = ccol;
                        if *in_0 == 0xa {
                            loop {
                                (*(*ctxt).input).line += 1;
                                (*(*ctxt).input).col = 1;
                                in_0 = in_0.offset(1);
                                if !(*in_0 == 0xa) {
                                    break;
                                }
                            }
                        } else {
                            if !(*in_0 == ']' as u8) {
                                break;
                            }
                            if *in_0.offset(1) == ']' as u8 && *in_0.offset(2) == '>' as u8 {
                                xmlFatalErr(ctxt, XML_ERR_MISPLACED_CDATA_END, 0 as *const i8);
                                (*(*ctxt).input).cur = in_0.offset(1);
                                return;
                            }
                            in_0 = in_0.offset(1);
                            (*(*ctxt).input).col += 1
                        }
                    }
                }
                nbchar = unsafe { in_0.offset_from((*(*ctxt).input).cur) as i32 };
                if nbchar > 0 {
                    if unsafe {
                        !(*ctxt).sax.is_null()
                            && (*(*ctxt).sax).ignorableWhitespace != (*(*ctxt).sax).characters
                            && (*(*(*ctxt).input).cur == 0x20
                                || 0x9 <= *(*(*ctxt).input).cur && *(*(*ctxt).input).cur <= 0xa
                                || *(*(*ctxt).input).cur == 0xd)
                    } {
                        let tmp_0: *const xmlChar = unsafe { (*(*ctxt).input).cur };
                        unsafe { (*(*ctxt).input).cur = in_0 };
                        if areBlanks(ctxt, tmp_0, nbchar, 0) != 0 {
                            if unsafe { (*(*ctxt).sax).ignorableWhitespace.is_some() } {
                                unsafe {
                                    (*(*ctxt).sax)
                                        .ignorableWhitespace
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        tmp_0,
                                        nbchar,
                                    );
                                }
                            }
                        } else {
                            unsafe {
                                if (*(*ctxt).sax).characters.is_some() {
                                    (*(*ctxt).sax)
                                        .characters
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        tmp_0,
                                        nbchar,
                                    );
                                }
                                if *(*ctxt).space == -1 {
                                    *(*ctxt).space = -2
                                }
                            }
                        }
                        unsafe {
                            line = (*(*ctxt).input).line;
                            col = (*(*ctxt).input).col
                        }
                    } else if !(safe_ctxt).sax.is_null() {
                        unsafe {
                            if (*(*ctxt).sax).characters.is_some() {
                                (*(*ctxt).sax)
                                    .characters
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    (*(*ctxt).input).cur,
                                    nbchar,
                                );
                            }
                            line = (*(*ctxt).input).line;
                            col = (*(*ctxt).input).col
                        }
                    }
                    /* something really bad happened in the SAX callback */
                    if (safe_ctxt).instate != XML_PARSER_CONTENT as i32 {
                        return;
                    }
                }
                unsafe {
                    (*(*ctxt).input).cur = in_0;
                    if *in_0 == 0xd {
                        in_0 = in_0.offset(1);
                        if *in_0 == 0xa {
                            (*(*ctxt).input).cur = in_0;
                            in_0 = in_0.offset(1);
                            (*(*ctxt).input).line += 1;
                            (*(*ctxt).input).col = 1;
                            current_block = 1917311967535052937;
                            /* while */
                        } else {
                            in_0 = in_0.offset(-1);
                            current_block = 1;
                        }
                    } else {
                        current_block = 1;
                    }
                }

                match current_block {
                    1 => {
                        if unsafe { *in_0 } == '<' as u8 {
                            return;
                        }
                        if unsafe { *in_0 } == '&' as u8 {
                            return;
                        }
                        SHRINK(ctxt);
                        GROW(ctxt);
                        if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
                            return;
                        }
                        in_0 = unsafe { (*(*ctxt).input).cur };
                    }
                    _ => {}
                }
                if unsafe { !(*in_0 >= 0x20 && *in_0 <= 0x7f || *in_0 == 0x9 || *in_0 == 0xa) } {
                    break;
                }
            }
        }
        nbchar = 0
    }
    unsafe {
        (*(*ctxt).input).line = line;
        (*(*ctxt).input).col = col;
    }
    xmlParseCharDataComplex(ctxt, cdata);
}
/* *
* xmlParseCharDataComplex:
* @ctxt:  an XML parser context
* @cdata:  int indicating whether we are within a CDATA section
*
* parse a CharData section.this is the fallback function
* of xmlParseCharData() when the parsing requires handling
* of non-ASCII characters.
*/
fn xmlParseCharDataComplex(ctxt: xmlParserCtxtPtr, cdata: i32) {
    let mut buf: [xmlChar; XML_PARSER_BIG_BUFFER_SIZE as usize + 5] =
        [0; XML_PARSER_BIG_BUFFER_SIZE as usize + 5];
    let mut nbchar: i32 = 0;
    let mut cur: i32 = 0;
    let mut l: i32 = 0;
    let mut count: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    SHRINK(ctxt);
    GROW(ctxt);
    cur = unsafe { xmlCurrentChar(ctxt, &mut l) };
    while cur != '<' as i32 && cur != '&' as i32 && IS_CHAR(cur) {
        /* test also done in xmlCurrentChar() */
        if cur == ']' as i32
            && unsafe {
                *(*(*ctxt).input).cur.offset(1) == ']' as u8
                    && *(*(*ctxt).input).cur.offset(2) == '>' as u8
            }
        {
            if cdata != 0 {
                break;
            }
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_MISPLACED_CDATA_END, 0 as *const i8);
            }
        }
        if l == 1 {
            let fresh84 = nbchar;
            nbchar = nbchar + 1;
            buf[fresh84 as usize] = cur as xmlChar
        } else {
            nbchar += unsafe {
                xmlCopyCharMultiByte(&mut *buf.as_mut_ptr().offset(nbchar as isize), cur)
            };
        }
        if nbchar >= XML_PARSER_BIG_BUFFER_SIZE as i32 {
            buf[nbchar as usize] = 0;
            /*
             * OK the segment is to be consumed as chars.
             */
            if !(safe_ctxt).sax.is_null() && (safe_ctxt).disableSAX == 0 {
                if areBlanks(ctxt, buf.as_mut_ptr(), nbchar, 0) != 0 {
                    unsafe {
                        if (*(*ctxt).sax).ignorableWhitespace.is_some() {
                            (*(*ctxt).sax)
                                .ignorableWhitespace
                                .expect("non-null function pointer")(
                                (*ctxt).userData,
                                buf.as_mut_ptr(),
                                nbchar,
                            );
                        }
                    }
                } else {
                    unsafe {
                        if (*(*ctxt).sax).characters.is_some() {
                            (*(*ctxt).sax)
                                .characters
                                .expect("non-null function pointer")(
                                (*ctxt).userData,
                                buf.as_mut_ptr(),
                                nbchar,
                            );
                        }
                        if (*(*ctxt).sax).characters != (*(*ctxt).sax).ignorableWhitespace
                            && *(*ctxt).space == -1
                        {
                            *(*ctxt).space = -2
                        }
                    }
                }
            }
            nbchar = 0;
            /* something really bad happened in the SAX callback */
            if (safe_ctxt).instate != XML_PARSER_CONTENT as i32 {
                return;
            }
        }
        count += 1;
        if count > 50 {
            SHRINK(ctxt);
            GROW(ctxt);
            count = 0;
            if (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32 {
                return;
            }
        }
        NEXTL(ctxt, l);
        unsafe {
            cur = xmlCurrentChar(ctxt, &mut l);
        }
    }
    if nbchar != 0 {
        buf[nbchar as usize] = 0;
        /*
         * OK the segment is to be consumed as chars.
         */
        if !(safe_ctxt).sax.is_null() && (safe_ctxt).disableSAX == 0 {
            if areBlanks(ctxt, buf.as_mut_ptr(), nbchar, 0) != 0 {
                unsafe {
                    if (*(*ctxt).sax).ignorableWhitespace.is_some() {
                        (*(*ctxt).sax)
                            .ignorableWhitespace
                            .expect("non-null function pointer")(
                            (*ctxt).userData,
                            buf.as_mut_ptr(),
                            nbchar,
                        );
                    }
                }
            } else {
                unsafe {
                    if (*(*ctxt).sax).characters.is_some() {
                        (*(*ctxt).sax)
                            .characters
                            .expect("non-null function pointer")(
                            (*ctxt).userData,
                            buf.as_mut_ptr(),
                            nbchar,
                        );
                    }
                    if (*(*ctxt).sax).characters != (*(*ctxt).sax).ignorableWhitespace
                        && *(*ctxt).space == -(1)
                    {
                        *(*ctxt).space = -(2)
                    }
                }
            }
        }
    }
    if cur != 0 && !IS_CHAR(cur) {
        /* Generate the error and skip the offending character */
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"PCDATA invalid Char value %d\n\x00" as *const u8 as *const i8,
                cur,
            );
        }
        NEXTL(ctxt, l);
    };
}
/* *
* xmlParseExternalID:
* @ctxt:  an XML parser context
* @publicID:  a xmlChar** receiving PubidLiteral
* @strict: indicate whether we should restrict parsing to only
*          production [75], see NOTE below
*
* Parse an External ID or a Public ID
*
* NOTE: Productions [75] and [83] interact badly since [75] can generate
*       'PUBLIC' S PubidLiteral S SystemLiteral
*
* [75] ExternalID ::= 'SYSTEM' S SystemLiteral
*                   | 'PUBLIC' S PubidLiteral S SystemLiteral
*
* [83] PublicID ::= 'PUBLIC' S PubidLiteral
*
* Returns the function returns SystemLiteral and in the second
*                case publicID receives PubidLiteral, is strict is off
*                it is possible to return NULL and have publicID set.
*/
fn CMP5(cur: *const u8, c1: char, c2: char, c3: char, c4: char, c5: char) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
    };
}

fn CMP6(cur: *const u8, c1: char, c2: char, c3: char, c4: char, c5: char, c6: char) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
            && *(cur as *mut u8).offset(5) == c6 as u8
    };
}

fn CMP7(
    cur: *const u8,
    c1: char,
    c2: char,
    c3: char,
    c4: char,
    c5: char,
    c6: char,
    c7: char,
) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
            && *(cur as *mut u8).offset(5) == c6 as u8
            && *(cur as *mut u8).offset(6) == c7 as u8
    };
}

fn CMP8(
    cur: *const u8,
    c1: char,
    c2: char,
    c3: char,
    c4: char,
    c5: char,
    c6: char,
    c7: char,
    c8: char,
) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
            && *(cur as *mut u8).offset(5) == c6 as u8
            && *(cur as *mut u8).offset(6) == c7 as u8
            && *(cur as *mut u8).offset(7) == c8 as u8
    };
}

fn CMP9(
    cur: *const u8,
    c1: char,
    c2: char,
    c3: char,
    c4: char,
    c5: char,
    c6: char,
    c7: char,
    c8: char,
    c9: char,
) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
            && *(cur as *mut u8).offset(5) == c6 as u8
            && *(cur as *mut u8).offset(6) == c7 as u8
            && *(cur as *mut u8).offset(7) == c8 as u8
            && *(cur as *mut u8).offset(8) == c9 as u8
    };
}

fn CMP10(
    cur: *const u8,
    c1: char,
    c2: char,
    c3: char,
    c4: char,
    c5: char,
    c6: char,
    c7: char,
    c8: char,
    c9: char,
    c10: char,
) -> bool {
    return unsafe {
        *(cur as *mut u8).offset(0) == c1 as u8
            && *(cur as *mut u8).offset(1) == c2 as u8
            && *(cur as *mut u8).offset(2) == c3 as u8
            && *(cur as *mut u8).offset(3) == c4 as u8
            && *(cur as *mut u8).offset(4) == c5 as u8
            && *(cur as *mut u8).offset(5) == c6 as u8
            && *(cur as *mut u8).offset(6) == c7 as u8
            && *(cur as *mut u8).offset(7) == c8 as u8
            && *(cur as *mut u8).offset(8) == c9 as u8
            && *(cur as *mut u8).offset(9) == c10 as u8
    };
}

pub fn xmlParseExternalID(
    ctxt: xmlParserCtxtPtr,
    publicID: *mut *mut xmlChar,
    strict: i32,
) -> *mut xmlChar {
    let safe_ctxt = unsafe { &mut *ctxt };
    let mut URI: *mut xmlChar = 0 as *mut xmlChar;
    SHRINK(ctxt);
    unsafe { *publicID = 0 as *mut xmlChar };
    if CMP6(
        unsafe { (*(*ctxt).input).cur },
        'S',
        'Y',
        'S',
        'T',
        'E',
        'M',
    ) {
        SKIP(ctxt, 6);
        if xmlSkipBlankChars(ctxt) == 0 {
            unsafe {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'SYSTEM\'\n\x00" as *const u8 as *const i8,
                );
            }
        }
        URI = xmlParseSystemLiteral(ctxt);
        if URI.is_null() {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_URI_REQUIRED, 0 as *const i8);
            }
        }
    } else if CMP6(
        unsafe { (*(*ctxt).input).cur },
        'P',
        'U',
        'B',
        'L',
        'I',
        'C',
    ) {
        SKIP(ctxt, 6);
        if xmlSkipBlankChars(ctxt) == 0 {
            unsafe {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'PUBLIC\'\n\x00" as *const u8 as *const i8,
                );
            }
        }
        unsafe {
            *publicID = xmlParsePubidLiteral(ctxt);
            if (*publicID).is_null() {
                xmlFatalErr(ctxt, XML_ERR_PUBID_REQUIRED, 0 as *const i8);
            }
        }
        if strict != 0 {
            /*
             * We don't handle [83] so "S SystemLiteral" is required.
             */
            if xmlSkipBlankChars(ctxt) == 0 {
                unsafe {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_SPACE_REQUIRED,
                        b"Space required after the Public Identifier\n\x00" as *const u8
                            as *const i8,
                    );
                }
            }
        } else {
            /*
             * We handle [83] so we return immediately, if
             * "S SystemLiteral" is not detected. We skip blanks if no
             * system literal was found, but this is harmless since we must
             * be at the end of a NotationDecl.
             */
            if xmlSkipBlankChars(ctxt) == 0 {
                return 0 as *mut xmlChar;
            }
            if unsafe { *(*(*ctxt).input).cur != '\'' as u8 && *(*(*ctxt).input).cur != '\"' as u8 }
            {
                return 0 as *mut xmlChar;
            }
        }
        URI = xmlParseSystemLiteral(ctxt);
        if URI.is_null() {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_URI_REQUIRED, 0 as *const i8);
            }
        }
    }
    return URI;
}
/* *
* xmlParseCommentComplex:
* @ctxt:  an XML parser context
* @buf:  the already parsed part of the buffer
* @len:  number of bytes in the buffer
* @size:  allocated size of the buffer
*
* Skip an XML (SGML) comment <!-- .... -->
*  The spec says that "For compatibility, the string "--" (double-hyphen)
*  must not occur within comments. "
* This is the slow routine in case the accelerator for ascii didn't work
*
* [15] Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
*/
fn xmlParseCommentComplex(
    ctxt: xmlParserCtxtPtr,
    mut buf: *mut xmlChar,
    mut len: size_t,
    mut size: size_t,
) {
    let safe_ctxt = unsafe { &mut *ctxt };
    let mut q: i32;
    let mut ql: i32 = 0;
    let mut r: i32;
    let mut rl: i32 = 0;
    let mut cur: i32;
    let mut l: i32 = 0;
    let mut count: size_t = 0;
    let mut inputid: i32;
    inputid = unsafe { (*(*ctxt).input).id };
    if buf.is_null() {
        len = 0;
        size = XML_PARSER_BUFFER_SIZE;
        buf = unsafe { xmlMallocAtomic_safe(size.wrapping_mul(size_of::<xmlChar>() as u64)) }
            as *mut xmlChar;
        if buf.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return;
        }
    }
    GROW(ctxt);
    /* Assure there's enough input data */
    unsafe {
        q = xmlCurrentChar(ctxt, &mut ql);
    }
    if !(q == 0) {
        if !IS_CHAR(q) {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INVALID_CHAR,
                b"xmlParseComment: invalid xmlChar value %d\n\x00" as *const u8 as *const i8,
                q,
            );
            unsafe { xmlFree_safe(buf as *mut ()) };
            return;
        }
        NEXTL(ctxt, ql);
        unsafe {
            r = xmlCurrentChar(ctxt, &mut rl);
        }
        if !(r == 0) {
            if !IS_CHAR(r) {
                xmlFatalErrMsgInt(
                    ctxt,
                    XML_ERR_INVALID_CHAR,
                    b"xmlParseComment: invalid xmlChar value %d\n\x00" as *const u8 as *const i8,
                    q,
                );
                unsafe { xmlFree_safe(buf as *mut ()) };
                return;
            }
            NEXTL(ctxt, rl);
            unsafe {
                cur = xmlCurrentChar(ctxt, &mut l);
            }
            if !(cur == 0) {
                while IS_CHAR(cur) && (cur != '>' as i32 || r != '-' as i32 || q != '-' as i32) {
                    if r == '-' as i32 && q == '-' as i32 {
                        unsafe {
                            xmlFatalErr(ctxt, XML_ERR_HYPHEN_IN_COMMENT, 0 as *const i8);
                        }
                    }
                    if len > XML_MAX_TEXT_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
                    {
                        xmlFatalErrMsgStr(
                            ctxt,
                            XML_ERR_COMMENT_NOT_FINISHED,
                            b"Comment too big found\x00" as *const u8 as *const i8,
                            0 as *const xmlChar,
                        );
                        unsafe { xmlFree_safe(buf as *mut ()) };
                        return;
                    }
                    if len + 5 >= size {
                        let mut new_buf: *mut xmlChar;
                        let mut new_size: size_t;
                        new_size = size * 2;
                        new_buf =
                            unsafe { xmlRealloc_safe(buf as *mut (), new_size) } as *mut xmlChar;
                        if new_buf.is_null() {
                            unsafe { xmlFree_safe(buf as *mut ()) };
                            unsafe {
                                xmlErrMemory(ctxt, 0 as *const i8);
                            }
                            return;
                        }
                        buf = new_buf;
                        size = new_size
                    }
                    if ql == 1 {
                        let fresh85 = len;
                        len += 1;
                        unsafe { *buf.offset(fresh85 as isize) = q as xmlChar }
                    } else {
                        len = unsafe {
                            (len as u64).wrapping_add(xmlCopyCharMultiByte(
                                &mut *buf.offset(len as isize),
                                q,
                            ) as u64) as size_t
                        };
                    }
                    q = r;
                    ql = rl;
                    r = cur;
                    rl = l;
                    count += 1;
                    if count > 50 {
                        SHRINK(ctxt);
                        GROW(ctxt);
                        count = 0;
                        if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
                            unsafe { xmlFree_safe(buf as *mut ()) };
                            return;
                        }
                    }
                    NEXTL(ctxt, l);
                    unsafe {
                        cur = xmlCurrentChar(ctxt, &mut l);
                    }
                    if cur == 0 {
                        SHRINK(ctxt);
                        GROW(ctxt);
                        cur = unsafe { xmlCurrentChar(ctxt, &mut l) };
                    }
                }
                unsafe {
                    *buf.offset(len as isize) = 0;
                }
                if cur == 0 {
                    xmlFatalErrMsgStr(
                        ctxt,
                        XML_ERR_COMMENT_NOT_FINISHED,
                        b"Comment not terminated \n<!--%.50s\n\x00" as *const u8 as *const i8,
                        buf,
                    );
                } else if !IS_CHAR(cur) {
                    xmlFatalErrMsgInt(
                        ctxt,
                        XML_ERR_INVALID_CHAR,
                        b"xmlParseComment: invalid xmlChar value %d\n\x00" as *const u8
                            as *const i8,
                        cur,
                    );
                } else {
                    unsafe {
                        if inputid != (*(*ctxt).input).id {
                            xmlFatalErrMsg(
                                ctxt,
                                XML_ERR_ENTITY_BOUNDARY,
                                b"Comment doesn\'t start and stop in the same entity\n\x00"
                                    as *const u8 as *const i8,
                            );
                        }
                    }
                    unsafe { xmlNextChar_safe(ctxt) };
                    unsafe {
                        if !(*ctxt).sax.is_null()
                            && (*(*ctxt).sax).comment.is_some()
                            && (*ctxt).disableSAX == 0
                        {
                            (*(*ctxt).sax).comment.expect("non-null function pointer")(
                                (*ctxt).userData,
                                buf,
                            );
                        }
                    }
                }
                unsafe { xmlFree_safe(buf as *mut ()) };
                return;
            }
        }
    }
    xmlFatalErrMsgStr(
        ctxt,
        XML_ERR_COMMENT_NOT_FINISHED,
        b"Comment not terminated\n\x00" as *const u8 as *const i8,
        0 as *const xmlChar,
    );
    unsafe { xmlFree_safe(buf as *mut ()) };
}
/* *
* xmlParseComment:
* @ctxt:  an XML parser context
*
* Skip an XML (SGML) comment <!-- .... -->
*  The spec says that "For compatibility, the string "--" (double-hyphen)
*  must not occur within comments. "
*
* [15] Comment ::= '<!--' ((Char - '-') | ('-' (Char - '-')))* '-->'
*/

pub fn xmlParseComment(ctxt: xmlParserCtxtPtr) {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut size: size_t = XML_PARSER_BUFFER_SIZE;
    let mut len: size_t = 0;
    let mut state: xmlParserInputState = XML_PARSER_START;
    let mut in_0: *const xmlChar;
    let mut nbchar: size_t = 0;
    let mut ccol: i32;
    let mut inputid: i32;
    let safe_ctxt = unsafe { &mut *ctxt };
    /*
     * Check that there is a comment right here.
     */
    if unsafe {
        *(*(*ctxt).input).cur != '<' as u8
            || *(*(*ctxt).input).cur.offset(1) != '!' as u8
            || *(*(*ctxt).input).cur.offset(2) != '-' as u8
            || *(*(*ctxt).input).cur.offset(3) != '-' as u8
    } {
        return;
    }
    state = (safe_ctxt).instate;
    (safe_ctxt).instate = XML_PARSER_COMMENT;
    unsafe {
        inputid = (*(*ctxt).input).id;
    }
    SKIP(ctxt, 4);
    SHRINK(ctxt);
    GROW(ctxt);
    /*
     * Accelerated common case where input don't need to be
     * modified before passing it to the handler.
     */
    in_0 = unsafe { (*(*ctxt).input).cur };
    loop {
        //@todo unsafe范围缩小
        unsafe {
            if *in_0 == 0xa {
                loop {
                    (*(*ctxt).input).line += 1;
                    (*(*ctxt).input).col = 1;
                    in_0 = in_0.offset(1);
                    if !(*in_0 == 0xa) {
                        break;
                    }
                }
            }
            loop {
                ccol = (*(*ctxt).input).col;
                while *in_0 > '-' as u8 && *in_0 <= 0x7f
                    || *in_0 >= 0x20 && *in_0 < '-' as u8
                    || *in_0 == 0x9
                {
                    in_0 = in_0.offset(1);
                    ccol += 1
                }
                (*(*ctxt).input).col = ccol;
                if *in_0 == 0xa {
                    loop {
                        (*(*ctxt).input).line += 1;
                        (*(*ctxt).input).col = 1;
                        in_0 = in_0.offset(1);
                        if !(*in_0 == 0xa) {
                            break;
                        }
                    }
                }
                nbchar = in_0.offset_from((*(*ctxt).input).cur) as size_t;
                /*
                 * save current set of data
                 */
                if nbchar > 0 {
                    if !(*ctxt).sax.is_null() && (*(*ctxt).sax).comment.is_some() {
                        if buf.is_null() {
                            if *in_0 == '-' as u8 && *in_0.offset(1) == '-' as u8 {
                                size = nbchar + 1;
                            } else {
                                size = XML_PARSER_BUFFER_SIZE + nbchar
                            }
                            buf = xmlMallocAtomic_safe(size * size_of::<xmlChar>() as u64)
                                as *mut xmlChar;
                            if buf.is_null() {
                                xmlErrMemory(ctxt, 0 as *const i8);
                                (*ctxt).instate = state;
                                return;
                            }
                            len = 0
                        } else if len + nbchar + 1 >= size {
                            let mut new_buf: *mut xmlChar;
                            size += len + nbchar + XML_PARSER_BUFFER_SIZE;
                            new_buf =
                                xmlRealloc_safe(buf as *mut (), size * size_of::<xmlChar>() as u64)
                                    as *mut xmlChar;
                            if new_buf.is_null() {
                                xmlFree_safe(buf as *mut ());
                                xmlErrMemory(ctxt, 0 as *const i8);
                                (*ctxt).instate = state;
                                return;
                            }
                            buf = new_buf
                        }
                        memcpy_safe(
                            &mut *buf.offset(len as isize) as *mut xmlChar as *mut (),
                            (*(*ctxt).input).cur as *const (),
                            nbchar,
                        );
                        len += nbchar;
                        *buf.offset(len as isize) = 0
                    }
                }
                if len > XML_MAX_TEXT_LENGTH && (*ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                    xmlFatalErrMsgStr(
                        ctxt,
                        XML_ERR_COMMENT_NOT_FINISHED,
                        b"Comment too big found\x00" as *const u8 as *const i8,
                        0 as *const xmlChar,
                    );
                    xmlFree_safe(buf as *mut ());
                    return;
                }
                (*(*ctxt).input).cur = in_0;
                if *in_0 == 0xa {
                    in_0 = in_0.offset(1);
                    (*(*ctxt).input).line += 1;
                    (*(*ctxt).input).col = 1
                }
                if *in_0 == 0xd {
                    in_0 = in_0.offset(1);
                    if *in_0 == 0xa {
                        (*(*ctxt).input).cur = in_0;
                        in_0 = in_0.offset(1);
                        (*(*ctxt).input).line += 1;
                        (*(*ctxt).input).col = 1;
                        break;
                        /* while */
                    } else {
                        in_0 = in_0.offset(-1)
                    }
                }
                SHRINK(ctxt);
                GROW(ctxt);
                if (*ctxt).instate == XML_PARSER_EOF as i32 {
                    xmlFree_safe(buf as *mut ());
                    return;
                }
                in_0 = (*(*ctxt).input).cur;
                if !(*in_0 == '-' as u8) {
                    break;
                }
                if *in_0.offset(1) == '-' as u8 {
                    if *in_0.offset(2) == '>' as u8 {
                        if (*(*ctxt).input).id != inputid {
                            xmlFatalErrMsg(
                                ctxt,
                                XML_ERR_ENTITY_BOUNDARY,
                                b"comment doesn\'t start and stop in the same entity\n\x00"
                                    as *const u8 as *const i8,
                            );
                        }
                        SKIP(ctxt, 3);
                        if !(*ctxt).sax.is_null()
                            && (*(*ctxt).sax).comment.is_some()
                            && (*ctxt).disableSAX == 0
                        {
                            if !buf.is_null() {
                                (*(*ctxt).sax).comment.expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    buf,
                                );
                            } else {
                                (*(*ctxt).sax).comment.expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    b"\x00" as *const u8 as *const i8 as *mut xmlChar,
                                );
                            }
                        }
                        if !buf.is_null() {
                            xmlFree_safe(buf as *mut ());
                        }
                        if (*ctxt).instate != XML_PARSER_EOF as i32 {
                            (*ctxt).instate = state
                        }
                        return;
                    }
                    if !buf.is_null() {
                        xmlFatalErrMsgStr(
                            ctxt,
                            XML_ERR_HYPHEN_IN_COMMENT,
                            b"Double hyphen within comment: <!--%.50s\n\x00" as *const u8
                                as *const i8,
                            buf,
                        );
                    } else {
                        xmlFatalErrMsgStr(
                            ctxt,
                            XML_ERR_HYPHEN_IN_COMMENT,
                            b"Double hyphen within comment\n\x00" as *const u8 as *const i8,
                            0 as *const xmlChar,
                        );
                    }
                    if (*ctxt).instate == XML_PARSER_EOF as i32 {
                        xmlFree_safe(buf as *mut ());
                        return;
                    }
                    in_0 = in_0.offset(1);
                    (*(*ctxt).input).col += 1
                }
                in_0 = in_0.offset(1);
                (*(*ctxt).input).col += 1
            }
            if !(*in_0 >= 0x20 && *in_0 <= 0x7f || *in_0 == 0x9 || *in_0 == 0xa) {
                break;
            }
        }
    }
    xmlParseCommentComplex(ctxt, buf, len, size);
    (safe_ctxt).instate = state;
}
/* *
* xmlParsePITarget:
* @ctxt:  an XML parser context
*
* parse the name of a PI
*
* [17] PITarget ::= Name - (('X' | 'x') ('M' | 'm') ('L' | 'l'))
*
* Returns the PITarget name or NULL
*/

pub fn xmlParsePITarget(ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut name: *const xmlChar;
    name = xmlParseName(ctxt);
    if unsafe {
        !name.is_null()
            && (*name.offset(0) == 'x' as u8 || *name.offset(0) == 'X' as u8)
            && (*name.offset(1) == 'm' as u8 || *name.offset(1) == 'M' as u8)
            && (*name.offset(2) == 'l' as u8 || *name.offset(2) == 'L' as u8)
    } {
        let mut i: i32 = 0;
        if unsafe {
            *name.offset(0) == 'x' as u8
                && *name.offset(1) == 'm' as u8
                && *name.offset(2) == 'l' as u8
                && *name.offset(3) == 0
        } {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_RESERVED_XML_NAME,
                b"XML declaration allowed only at the start of the document\n\x00" as *const u8
                    as *const i8,
            );
            return name;
        } else {
            unsafe {
                if *name.offset(3) == 0 {
                    xmlFatalErr(ctxt, XML_ERR_RESERVED_XML_NAME, 0 as *const i8);
                    return name;
                }
            }
        }
        i = 0;
        while unsafe { !xmlW3CPIs[i as usize].is_null() } {
            if unsafe { xmlStrEqual_safe(name, unsafe { xmlW3CPIs[i as usize] as *const xmlChar }) }
                != 0
            {
                return name;
            }
            i += 1
        }
        xmlWarningMsg(
            ctxt,
            XML_ERR_RESERVED_XML_NAME,
            b"xmlParsePITarget: invalid name prefix \'xml\'\n\x00" as *const u8 as *const i8,
            0 as *const xmlChar,
            0 as *const xmlChar,
        );
    }
    if !name.is_null() && !unsafe { xmlStrchr_safe(name, ':' as xmlChar) }.is_null() {
        xmlNsErr(
            ctxt,
            XML_NS_ERR_COLON,
            b"colons are forbidden from PI names \'%s\'\n\x00" as *const u8 as *const i8,
            name,
            0 as *const xmlChar,
            0 as *const xmlChar,
        );
    }
    return name;
}
/* *
* xmlParseCatalogPI:
* @ctxt:  an XML parser context
* @catalog:  the PI value string
*
* parse an XML Catalog Processing Instruction.
*
* <?oasis-xml-catalog catalog="http://example.com/catalog.xml"?>
*
* Occurs only if allowed by the user and if happening in the Misc
* part of the document before any doctype information
* This will add the given catalog to the parsing context in order
* to be used if there is a resolution need further down in the document
 */

#[cfg(HAVE_parser_LIBXML_CATALOG_ENABLED)]
fn xmlParseCatalogPI(ctxt: xmlParserCtxtPtr, catalog: *const xmlChar) {
    let mut URL: *mut xmlChar = 0 as *mut xmlChar;
    let mut tmp: *const xmlChar;
    let mut base: *const xmlChar;
    let mut marker: xmlChar = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    tmp = catalog;
    while IS_BLANK_CH(tmp) {
        unsafe { tmp = tmp.offset(1) }
    }
    if !(unsafe {
        xmlStrncmp_safe(
            tmp,
            b"catalog\x00" as *const u8 as *const i8 as *mut xmlChar,
            7,
        )
    } != 0)
    {
        unsafe {
            tmp = tmp.offset(7);
            while IS_BLANK_CH(tmp) {
                tmp = tmp.offset(1)
            }
            if *tmp != '=' as u8 {
                return;
            }
            tmp = tmp.offset(1);
            while IS_BLANK_CH(tmp) {
                tmp = tmp.offset(1)
            }
            marker = *tmp;
        }

        if !(marker != '\'' as u8 && marker != '\"' as u8) {
            unsafe {
                tmp = tmp.offset(1);
                base = tmp;
                while *tmp != 0 && *tmp != marker {
                    tmp = tmp.offset(1)
                }
            }
            if !(unsafe { *tmp } == 0) {
                unsafe {
                    URL = xmlStrndup_safe(base, tmp.offset_from(base) as i32);
                    tmp = tmp.offset(1);
                    while IS_BLANK_CH(tmp) {
                        tmp = tmp.offset(1)
                    }
                }
                if !(unsafe { *tmp } != 0) {
                    if !URL.is_null() {
                        (safe_ctxt).catalogs =
                            unsafe { xmlCatalogAddLocal_safe((safe_ctxt).catalogs, URL) };
                        unsafe { xmlFree_safe(URL as *mut ()) };
                    }
                    return;
                }
            }
        }
    }
    xmlWarningMsg(
        ctxt,
        XML_WAR_CATALOG_PI,
        b"Catalog PI syntax error: %s\n\x00" as *const u8 as *const i8,
        catalog,
        0 as *const xmlChar,
    );
    if !URL.is_null() {
        unsafe { xmlFree_safe(URL as *mut ()) };
    };
}
/* *
* xmlParsePI:
* @ctxt:  an XML parser context
*
* parse an XML Processing Instruction.
*
* [16] PI ::= '<?' PITarget (S (Char* - (Char* '?>' Char*)))? '?>'
*
* The processing is transferred to SAX once parsed.
*/

pub fn xmlParsePI(ctxt: xmlParserCtxtPtr) {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: size_t = 0;
    let mut size: size_t = XML_PARSER_BUFFER_SIZE;
    let mut cur: i32 = 0;
    let mut l: i32 = 0;
    let mut target: *const xmlChar;
    let mut state: xmlParserInputState = XML_PARSER_START;
    let mut count: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*(*ctxt).input).cur == '<' as u8 && *(*(*ctxt).input).cur.offset(1) == '?' as u8 }
    {
        let inputid: i32 = unsafe { (*(*ctxt).input).id };
        state = (safe_ctxt).instate;
        (safe_ctxt).instate = XML_PARSER_PI;
        /*
         * this is a Processing Instruction.
         */
        SKIP(ctxt, 2);
        SHRINK(ctxt);
        /*
         * Parse the target name and check for special support like
         * namespace.
         */
        target = xmlParsePITarget(ctxt);
        if !target.is_null() {
            if unsafe {
                *(*(*ctxt).input).cur == '?' as u8 && *(*(*ctxt).input).cur.offset(1) == '>' as u8
            } {
                if inputid != unsafe { (*(*ctxt).input).id } {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ENTITY_BOUNDARY,
                        b"PI declaration doesn\'t start and stop in the same entity\n\x00"
                            as *const u8 as *const i8,
                    );
                }
                SKIP(ctxt, 2);
                /*
                 * SAX: PI detected.
                 */
                unsafe {
                    if !(*ctxt).sax.is_null()
                        && (*ctxt).disableSAX == 0
                        && (*(*ctxt).sax).processingInstruction.is_some()
                    {
                        (*(*ctxt).sax)
                            .processingInstruction
                            .expect("non-null function pointer")(
                            (*ctxt).userData,
                            target,
                            0 as *const xmlChar,
                        );
                    }
                }
                if (safe_ctxt).instate != XML_PARSER_EOF as i32 {
                    (safe_ctxt).instate = state
                }
                return;
            }
            buf =
                unsafe { xmlMallocAtomic_safe(size * size_of::<xmlChar>() as u64) } as *mut xmlChar;
            if buf.is_null() {
                unsafe {
                    xmlErrMemory(ctxt, 0 as *const i8);
                }
                (safe_ctxt).instate = state;
                return;
            }
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"ParsePI: PI %s space expected\n\x00" as *const u8 as *const i8,
                    target,
                );
            }
            unsafe {
                cur = xmlCurrentChar(ctxt, &mut l);
            }
            while IS_CHAR(cur)
                && unsafe {
                    (cur != '?' as i32 || *(*(*ctxt).input).cur.offset(1) as i32 != '>' as i32)
                }
            {
                if len + 5 >= size {
                    let mut tmp: *mut xmlChar;
                    let new_size: size_t = size * 2;
                    tmp = unsafe { xmlRealloc_safe(buf as *mut (), new_size) } as *mut xmlChar;
                    if tmp.is_null() {
                        unsafe {
                            xmlErrMemory(ctxt, 0 as *const i8);
                        }
                        unsafe { xmlFree_safe(buf as *mut ()) };
                        (safe_ctxt).instate = state;
                        return;
                    }
                    buf = tmp;
                    size = new_size
                }
                count += 1;
                if count > 50 {
                    SHRINK(ctxt);
                    GROW(ctxt);
                    if (safe_ctxt).instate == XML_PARSER_EOF as i32 {
                        unsafe { xmlFree_safe(buf as *mut ()) };
                        return;
                    }
                    count = 0;
                    if len > XML_MAX_TEXT_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0
                    {
                        xmlFatalErrMsgStr(
                            ctxt,
                            XML_ERR_PI_NOT_FINISHED,
                            b"PI %s too big found\x00" as *const u8 as *const i8,
                            target,
                        );
                        unsafe { xmlFree_safe(buf as *mut ()) };
                        (safe_ctxt).instate = state;
                        return;
                    }
                }
                if l == 1 {
                    let fresh86 = len;
                    len += 1;
                    unsafe { *buf.offset(fresh86 as isize) = cur as xmlChar };
                } else {
                    len = unsafe {
                        (len as u64)
                            .wrapping_add(
                                xmlCopyCharMultiByte(&mut *buf.offset(len as isize), cur) as u64
                            ) as size_t
                    };
                }
                NEXTL(ctxt, l);
                unsafe {
                    cur = xmlCurrentChar(ctxt, &mut l);
                }
                if cur == 0 {
                    SHRINK(ctxt);
                    GROW(ctxt);
                    cur = unsafe { xmlCurrentChar(ctxt, &mut l) };
                }
            }
            if len > XML_MAX_TEXT_LENGTH && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_PI_NOT_FINISHED,
                    b"PI %s too big found\x00" as *const u8 as *const i8,
                    target,
                );
                unsafe { xmlFree_safe(buf as *mut ()) };
                (safe_ctxt).instate = state;
                return;
            }
            unsafe { *buf.offset(len as isize) = 0 };
            if cur != '?' as i32 {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_PI_NOT_FINISHED,
                    b"ParsePI: PI %s never end ...\n\x00" as *const u8 as *const i8,
                    target,
                );
            } else {
                unsafe {
                    if inputid != (*(*ctxt).input).id {
                        xmlFatalErrMsg(
                            ctxt,
                            XML_ERR_ENTITY_BOUNDARY,
                            b"PI declaration doesn\'t start and stop in the same entity\n\x00"
                                as *const u8 as *const i8,
                        );
                    }
                    SKIP(ctxt, 2);
                }

                match () {
                    #[cfg(HAVE_parser_LIBXML_CATALOG_ENABLED)]
                    _ => {
                        if (state == XML_PARSER_MISC || state == XML_PARSER_START)
                            && unsafe {
                                xmlStrEqual_safe(
                                    target,
                                    b"oasis-xml-catalog\x00" as *const u8 as *const i8
                                        as *const xmlChar,
                                )
                            } != 0
                        {
                            let allow: xmlCatalogAllow = unsafe { xmlCatalogGetDefaults_safe() };
                            if allow == XML_CATA_ALLOW_DOCUMENT || allow == XML_CATA_ALLOW_ALL {
                                xmlParseCatalogPI(ctxt, buf);
                            }
                        }
                    }
                    #[cfg(not(HAVE_parser_LIBXML_CATALOG_ENABLED))]
                    _ => {}
                };

                /*
                 * SAX: PI detected.
                 */
                unsafe {
                    if !(*ctxt).sax.is_null()
                        && (*ctxt).disableSAX == 0
                        && (*(*ctxt).sax).processingInstruction.is_some()
                    {
                        (*(*ctxt).sax)
                            .processingInstruction
                            .expect("non-null function pointer")(
                            (*ctxt).userData, target, buf
                        );
                    }
                }
            }
            unsafe { xmlFree_safe(buf as *mut ()) };
        } else {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_PI_NOT_STARTED, 0 as *const i8);
            }
        }
        if (safe_ctxt).instate != XML_PARSER_EOF {
            (safe_ctxt).instate = state
        }
    };
}
/* *
* xmlParseNotationDecl:
* @ctxt:  an XML parser context
*
* parse a notation declaration
*
* [82] NotationDecl ::= '<!NOTATION' S Name S (ExternalID |  PublicID) S? '>'
*
* Hence there is actually 3 choices:
*     'PUBLIC' S PubidLiteral
*     'PUBLIC' S PubidLiteral S SystemLiteral
* and 'SYSTEM' S SystemLiteral
*
* See the NOTE on xmlParseExternalID().
*/

pub fn xmlParseNotationDecl(ctxt: xmlParserCtxtPtr) {
    let mut name: *const xmlChar;
    let mut Pubid: *mut xmlChar = 0 as *mut xmlChar;
    let mut Systemid: *mut xmlChar;
    if CMP10(
        unsafe { (*(*ctxt).input).cur },
        '<',
        '!',
        'N',
        'O',
        'T',
        'A',
        'T',
        'I',
        'O',
        'N',
    ) {
        let inputid: i32 = unsafe { (*(*ctxt).input).id };
        SHRINK(ctxt);
        SKIP(ctxt, 10);
        if xmlSkipBlankChars(ctxt) == 0 {
            unsafe {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'<!NOTATION\'\n\x00" as *const u8 as *const i8,
                );
            }
            return;
        }
        name = xmlParseName(ctxt);
        if name.is_null() {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_NOTATION_NOT_STARTED, 0 as *const i8);
            }
            return;
        }
        if !unsafe { xmlStrchr_safe(name, ':' as xmlChar) }.is_null() {
            xmlNsErr(
                ctxt,
                XML_NS_ERR_COLON,
                b"colons are forbidden from notation names \'%s\'\n\x00" as *const u8 as *const i8,
                name,
                0 as *const xmlChar,
                0 as *const xmlChar,
            );
        }
        if xmlSkipBlankChars(ctxt) == 0 {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_SPACE_REQUIRED,
                b"Space required after the NOTATION name\'\n\x00" as *const u8 as *const i8,
            );
            return;
        }
        /*
         * Parse the IDs.
         */
        Systemid = xmlParseExternalID(ctxt, &mut Pubid, 0);
        xmlSkipBlankChars(ctxt);
        unsafe {
            if *(*(*ctxt).input).cur == '>' as u8 {
                if inputid != (*(*ctxt).input).id {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ENTITY_BOUNDARY,
                        b"Notation declaration doesn\'t start and stop in the same entity\n\x00"
                            as *const u8 as *const i8,
                    );
                }
                xmlNextChar_safe(ctxt);
                if !(*ctxt).sax.is_null()
                    && (*ctxt).disableSAX == 0
                    && (*(*ctxt).sax).notationDecl.is_some()
                {
                    (*(*ctxt).sax)
                        .notationDecl
                        .expect("non-null function pointer")(
                        (*ctxt).userData,
                        name,
                        Pubid,
                        Systemid,
                    );
                }
            } else {
                xmlFatalErr(ctxt, XML_ERR_NOTATION_NOT_FINISHED, 0 as *const i8);
            }
        }

        if !Systemid.is_null() {
            unsafe { xmlFree_safe(Systemid as *mut ()) };
        }
        if !Pubid.is_null() {
            unsafe { xmlFree_safe(Pubid as *mut ()) };
        }
    };
}
/* *
* xmlParseEntityDecl:
* @ctxt:  an XML parser context
*
* parse <!ENTITY declarations
*
* [70] EntityDecl ::= GEDecl | PEDecl
*
* [71] GEDecl ::= '<!ENTITY' S Name S EntityDef S? '>'
*
* [72] PEDecl ::= '<!ENTITY' S '%' S Name S PEDef S? '>'
*
* [73] EntityDef ::= EntityValue | (ExternalID NDataDecl?)
*
* [74] PEDef ::= EntityValue | ExternalID
*
* [76] NDataDecl ::= S 'NDATA' S Name
*
* [ VC: Notation Declared ]
* The Name must match the declared name of a notation.
*/

pub fn xmlParseEntityDecl(ctxt: xmlParserCtxtPtr) {
    let mut name: *const xmlChar = 0 as *const xmlChar;
    let mut value: *mut xmlChar = 0 as *mut xmlChar;
    let mut URI: *mut xmlChar = 0 as *mut xmlChar;
    let mut literal: *mut xmlChar = 0 as *mut xmlChar;
    let mut ndata: *const xmlChar = 0 as *const xmlChar;
    let mut isParameter: i32 = 0;
    let mut orig: *mut xmlChar = 0 as *mut xmlChar;
    /* GROW; done in the caller */
    //@todo 削减unsafe范围
    unsafe {
        if CMP8((*(*ctxt).input).cur, '<', '!', 'E', 'N', 'T', 'I', 'T', 'Y') {
            let inputid: i32 = (*(*ctxt).input).id;
            SHRINK(ctxt);
            SKIP(ctxt, 8);
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'<!ENTITY\'\n\x00" as *const u8 as *const i8,
                );
            }
            if *(*(*ctxt).input).cur == '%' as u8 {
                xmlNextChar_safe(ctxt);
                if xmlSkipBlankChars(ctxt) == 0 {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_SPACE_REQUIRED,
                        b"Space required after \'%%\'\n\x00" as *const u8 as *const i8,
                    );
                }
                isParameter = 1
            }
            name = xmlParseName(ctxt);
            if name.is_null() {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_NAME_REQUIRED,
                    b"xmlParseEntityDecl: no name\n\x00" as *const u8 as *const i8,
                );
                return;
            }
            if !xmlStrchr_safe(name, ':' as xmlChar).is_null() {
                xmlNsErr(
                    ctxt,
                    XML_NS_ERR_COLON,
                    b"colons are forbidden from entities names \'%s\'\n\x00" as *const u8
                        as *const i8,
                    name,
                    0 as *const xmlChar,
                    0 as *const xmlChar,
                );
            }
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after the entity name\n\x00" as *const u8 as *const i8,
                );
            }
            (*ctxt).instate = XML_PARSER_ENTITY_DECL;
            /*
             * handle the various case of definitions...
             */
            if isParameter != 0 {
                if *(*(*ctxt).input).cur == '\"' as u8 || *(*(*ctxt).input).cur == '\'' as u8 {
                    value = xmlParseEntityValue(ctxt, &mut orig);
                    if !value.is_null() {
                        if !(*ctxt).sax.is_null()
                            && (*ctxt).disableSAX == 0
                            && (*(*ctxt).sax).entityDecl.is_some()
                        {
                            (*(*ctxt).sax)
                                .entityDecl
                                .expect("non-null function pointer")(
                                (*ctxt).userData,
                                name,
                                XML_INTERNAL_PARAMETER_ENTITY as i32,
                                0 as *const xmlChar,
                                0 as *const xmlChar,
                                value,
                            );
                        }
                    }
                } else {
                    URI = xmlParseExternalID(ctxt, &mut literal, 1);
                    if URI.is_null() && literal.is_null() {
                        xmlFatalErr(ctxt, XML_ERR_VALUE_REQUIRED, 0 as *const i8);
                    }
                    if !URI.is_null() {
                        let mut uri: xmlURIPtr;
                        uri = xmlParseURI_safe(URI as *const i8);
                        if uri.is_null() {
                            xmlErrMsgStr(
                                ctxt,
                                XML_ERR_INVALID_URI,
                                b"Invalid URI: %s\n\x00" as *const u8 as *const i8,
                                URI,
                            );
                            /*
                             * This really ought to be a well formedness error
                             * but the XML Core WG decided otherwise c.f. issue
                             * E26 of the XML erratas.
                             */
                        } else {
                            if !(*uri).fragment.is_null() {
                                /*
                                 * Okay this is foolish to block those but not
                                 * invalid URIs.
                                 */
                                xmlFatalErr(ctxt, XML_ERR_URI_FRAGMENT, 0 as *const i8);
                            } else if !(*ctxt).sax.is_null()
                                && (*ctxt).disableSAX == 0
                                && (*(*ctxt).sax).entityDecl.is_some()
                            {
                                (*(*ctxt).sax)
                                    .entityDecl
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    name,
                                    XML_EXTERNAL_PARAMETER_ENTITY as i32,
                                    literal,
                                    URI,
                                    0 as *mut xmlChar,
                                );
                            }
                            xmlFreeURI_safe(uri);
                        }
                    }
                }
            } else if *(*(*ctxt).input).cur == '\"' as u8 || *(*(*ctxt).input).cur == '\'' as u8 {
                value = xmlParseEntityValue(ctxt, &mut orig);
                if !(*ctxt).sax.is_null()
                    && (*ctxt).disableSAX == 0
                    && (*(*ctxt).sax).entityDecl.is_some()
                {
                    (*(*ctxt).sax)
                        .entityDecl
                        .expect("non-null function pointer")(
                        (*ctxt).userData,
                        name,
                        XML_INTERNAL_GENERAL_ENTITY as i32,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                        value,
                    );
                }
                /*
                 * For expat compatibility in SAX mode.
                 */
                if (*ctxt).myDoc.is_null()
                    || xmlStrEqual_safe(
                        (*(*ctxt).myDoc).version,
                        b"SAX compatibility mode document\x00" as *const u8 as *const i8
                            as *mut xmlChar,
                    ) != 0
                {
                    if (*ctxt).myDoc.is_null() {
                        (*ctxt).myDoc = xmlNewDoc_safe(
                            b"SAX compatibility mode document\x00" as *const u8 as *const i8
                                as *mut xmlChar,
                        );
                        if (*ctxt).myDoc.is_null() {
                            xmlErrMemory(ctxt, b"New Doc failed\x00" as *const u8 as *const i8);
                            return;
                        }
                        (*(*ctxt).myDoc).properties = XML_DOC_INTERNAL as i32
                    }
                    if (*(*ctxt).myDoc).intSubset.is_null() {
                        (*(*ctxt).myDoc).intSubset = xmlNewDtd(
                            (*ctxt).myDoc,
                            b"fake\x00" as *const u8 as *const i8 as *mut xmlChar,
                            0 as *const xmlChar,
                            0 as *const xmlChar,
                        )
                    }
                    xmlSAX2EntityDecl(
                        ctxt as *mut (),
                        name,
                        XML_INTERNAL_GENERAL_ENTITY as i32,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                        value,
                    );
                }
            } else {
                URI = xmlParseExternalID(ctxt, &mut literal, 1);
                if URI.is_null() && literal.is_null() {
                    xmlFatalErr(ctxt, XML_ERR_VALUE_REQUIRED, 0 as *const i8);
                }
                if !URI.is_null() {
                    let mut uri_0: xmlURIPtr;
                    uri_0 = xmlParseURI_safe(URI as *const i8);
                    if uri_0.is_null() {
                        xmlErrMsgStr(
                            ctxt,
                            XML_ERR_INVALID_URI,
                            b"Invalid URI: %s\n\x00" as *const u8 as *const i8,
                            URI,
                        );
                        /*
                         * This really ought to be a well formedness error
                         * but the XML Core WG decided otherwise c.f. issue
                         * E26 of the XML erratas.
                         */
                    } else {
                        if !(*uri_0).fragment.is_null() {
                            /*
                             * Okay this is foolish to block those but not
                             * invalid URIs.
                             */
                            xmlFatalErr(ctxt, XML_ERR_URI_FRAGMENT, 0 as *const i8);
                        }
                        xmlFreeURI_safe(uri_0);
                    }
                }
                if *(*(*ctxt).input).cur != '>' as u8 && xmlSkipBlankChars(ctxt) == 0 {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_SPACE_REQUIRED,
                        b"Space required before \'NDATA\'\n\x00" as *const u8 as *const i8,
                    );
                }
                if CMP5((*(*ctxt).input).cur, 'N', 'D', 'A', 'T', 'A') {
                    SKIP(ctxt, 5);
                    if xmlSkipBlankChars(ctxt) == 0 {
                        xmlFatalErrMsg(
                            ctxt,
                            XML_ERR_SPACE_REQUIRED,
                            b"Space required after \'NDATA\'\n\x00" as *const u8 as *const i8,
                        );
                    }
                    ndata = xmlParseName(ctxt);
                    if !(*ctxt).sax.is_null()
                        && (*ctxt).disableSAX == 0
                        && (*(*ctxt).sax).unparsedEntityDecl.is_some()
                    {
                        (*(*ctxt).sax)
                            .unparsedEntityDecl
                            .expect("non-null function pointer")(
                            (*ctxt).userData,
                            name,
                            literal,
                            URI,
                            ndata,
                        );
                    }
                } else {
                    if !(*ctxt).sax.is_null()
                        && (*ctxt).disableSAX == 0
                        && (*(*ctxt).sax).entityDecl.is_some()
                    {
                        (*(*ctxt).sax)
                            .entityDecl
                            .expect("non-null function pointer")(
                            (*ctxt).userData,
                            name,
                            XML_EXTERNAL_GENERAL_PARSED_ENTITY as i32,
                            literal,
                            URI,
                            0 as *mut xmlChar,
                        );
                    }
                    /*
                     * For expat compatibility in SAX mode.
                     * assuming the entity replacement was asked for
                     */
                    if (*ctxt).replaceEntities != 0
                        && ((*ctxt).myDoc.is_null()
                            || xmlStrEqual_safe(
                                (*(*ctxt).myDoc).version,
                                b"SAX compatibility mode document\x00" as *const u8 as *const i8
                                    as *mut xmlChar,
                            ) != 0)
                    {
                        if (*ctxt).myDoc.is_null() {
                            (*ctxt).myDoc = xmlNewDoc_safe(
                                b"SAX compatibility mode document\x00" as *const u8 as *const i8
                                    as *mut xmlChar,
                            );
                            if (*ctxt).myDoc.is_null() {
                                xmlErrMemory(ctxt, b"New Doc failed\x00" as *const u8 as *const i8);
                                return;
                            }
                            (*(*ctxt).myDoc).properties = XML_DOC_INTERNAL as i32
                        }
                        if (*(*ctxt).myDoc).intSubset.is_null() {
                            (*(*ctxt).myDoc).intSubset = xmlNewDtd(
                                (*ctxt).myDoc,
                                b"fake\x00" as *const u8 as *const i8 as *mut xmlChar,
                                0 as *const xmlChar,
                                0 as *const xmlChar,
                            )
                        }
                        xmlSAX2EntityDecl(
                            ctxt as *mut (),
                            name,
                            XML_EXTERNAL_GENERAL_PARSED_ENTITY as i32,
                            literal,
                            URI,
                            0 as *mut xmlChar,
                        );
                    }
                }
            }
            if !((*ctxt).instate == XML_PARSER_EOF) {
                xmlSkipBlankChars(ctxt);
                if *(*(*ctxt).input).cur != '>' as u8 {
                    xmlFatalErrMsgStr(
                        ctxt,
                        XML_ERR_ENTITY_NOT_FINISHED,
                        b"xmlParseEntityDecl: entity %s not terminated\n\x00" as *const u8
                            as *const i8,
                        name,
                    );
                    xmlHaltParser(ctxt);
                } else {
                    if inputid != (*(*ctxt).input).id {
                        xmlFatalErrMsg(
                            ctxt,
                            XML_ERR_ENTITY_BOUNDARY,
                            b"Entity declaration doesn\'t start and stop in the same entity\n\x00"
                                as *const u8 as *const i8,
                        );
                    }
                    xmlNextChar_safe(ctxt);
                }
                if !orig.is_null() {
                    /*
                     * Ugly mechanism to save the raw entity value.
                     */
                    let mut cur: xmlEntityPtr = 0 as xmlEntityPtr;
                    if isParameter != 0 {
                        if !(*ctxt).sax.is_null() && (*(*ctxt).sax).getParameterEntity.is_some() {
                            cur = (*(*ctxt).sax)
                                .getParameterEntity
                                .expect("non-null function pointer")(
                                (*ctxt).userData, name
                            )
                        }
                    } else {
                        if !(*ctxt).sax.is_null() && (*(*ctxt).sax).getEntity.is_some() {
                            cur = (*(*ctxt).sax).getEntity.expect("non-null function pointer")(
                                (*ctxt).userData,
                                name,
                            )
                        }
                        if cur.is_null() && (*ctxt).userData == ctxt as *mut () {
                            cur = xmlSAX2GetEntity_safe(ctxt as *mut (), name)
                        }
                    }
                    if !cur.is_null() && (*cur).orig.is_null() {
                        (*cur).orig = orig;
                        orig = 0 as *mut xmlChar
                    }
                }
            }
            if !value.is_null() {
                xmlFree_safe(value as *mut ());
            }
            if !URI.is_null() {
                xmlFree_safe(URI as *mut ());
            }
            if !literal.is_null() {
                xmlFree_safe(literal as *mut ());
            }
            if !orig.is_null() {
                xmlFree_safe(orig as *mut ());
            }
        };
    }
}
/* *
* xmlParseDefaultDecl:
* @ctxt:  an XML parser context
* @value:  Receive a possible fixed default value for the attribute
*
* Parse an attribute default declaration
*
* [60] DefaultDecl ::= '#REQUIRED' | '#IMPLIED' | (('#FIXED' S)? AttValue)
*
* [ VC: Required Attribute ]
* if the default declaration is the keyword #REQUIRED, then the
* attribute must be specified for all elements of the type in the
* attribute-list declaration.
*
* [ VC: Attribute Default Legal ]
* The declared default value must meet the lexical constraints of
* the declared attribute type c.f. xmlValidateAttributeDecl()
*
* [ VC: Fixed Attribute Default ]
* if an attribute has a default value declared with the #FIXED
* keyword, instances of that attribute must match the default value.
*
* [ WFC: No < in Attribute Values ]
* handled in xmlParseAttValue()
*
* returns: XML_ATTRIBUTE_NONE, XML_ATTRIBUTE_REQUIRED, XML_ATTRIBUTE_IMPLIED
*          or XML_ATTRIBUTE_FIXED.
*/

pub fn xmlParseDefaultDecl(ctxt: xmlParserCtxtPtr, value: *mut *mut xmlChar) -> i32 {
    let mut val: i32 = 0;
    let mut ret: *mut xmlChar = 0 as *mut xmlChar;
    //@todo 削减unsafe范围
    unsafe {
        *value = 0 as *mut xmlChar;
        if CMP9(
            (*(*ctxt).input).cur,
            '#',
            'R',
            'E',
            'Q',
            'U',
            'I',
            'R',
            'E',
            'D',
        ) {
            SKIP(ctxt, 9);
            return XML_ATTRIBUTE_REQUIRED as i32;
        }
        if CMP8((*(*ctxt).input).cur, '#', 'I', 'M', 'P', 'L', 'I', 'E', 'D') {
            SKIP(ctxt, 8);
            return XML_ATTRIBUTE_IMPLIED as i32;
        }
        val = XML_ATTRIBUTE_NONE as i32;
        if CMP6((*(*ctxt).input).cur, '#', 'F', 'I', 'X', 'E', 'D') {
            SKIP(ctxt, 6);
            val = XML_ATTRIBUTE_FIXED as i32;
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'#FIXED\'\n\x00" as *const u8 as *const i8,
                );
            }
        }
        ret = xmlParseAttValue(ctxt);
        (*ctxt).instate = XML_PARSER_DTD;
        if ret.is_null() {
            xmlFatalErrMsg(
                ctxt,
                (*ctxt).errNo as xmlParserErrors,
                b"Attribute default value declaration error\n\x00" as *const u8 as *const i8,
            );
        } else {
            *value = ret
        }
    }
    return val;
}
/* *
* xmlParseNotationType:
* @ctxt:  an XML parser context
*
* parse an Notation attribute type.
*
* Note: the leading 'NOTATION' S part has already being parsed...
*
* [58] NotationType ::= 'NOTATION' S '(' S? Name (S? '|' S? Name)* S? ')'
*
* [ VC: Notation Attributes ]
* Values of this type must match one of the notation names included
* in the declaration; all notation names in the declaration must be declared.
*
* Returns: the notation attribute tree built while parsing
*/

pub fn xmlParseNotationType(ctxt: xmlParserCtxtPtr) -> xmlEnumerationPtr {
    let mut name: *const xmlChar;
    let mut ret: xmlEnumerationPtr = 0 as xmlEnumerationPtr;
    let mut last: xmlEnumerationPtr = 0 as xmlEnumerationPtr;
    let mut cur: xmlEnumerationPtr;
    let mut tmp: xmlEnumerationPtr;
    let safe_ctxt = unsafe { &mut *ctxt };
    unsafe {
        if *(*(*ctxt).input).cur != '(' as u8 {
            xmlFatalErr(ctxt, XML_ERR_NOTATION_NOT_STARTED, 0 as *const i8);
            return 0 as xmlEnumerationPtr;
        }
    }
    SHRINK(ctxt);
    loop {
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        name = xmlParseName(ctxt);
        if name.is_null() {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_NAME_REQUIRED,
                b"Name expected in NOTATION declaration\n\x00" as *const u8 as *const i8,
            );
            unsafe { xmlFreeEnumeration_safe(ret) };
            return 0 as xmlEnumerationPtr;
        }
        tmp = ret;
        while !tmp.is_null() {
            if unsafe { xmlStrEqual_safe(name, unsafe { (*tmp).name }) } != 0 {
                xmlValidityError(
                    ctxt,
                    XML_DTD_DUP_TOKEN,
                    b"standalone: attribute notation value token %s duplicated\n\x00" as *const u8
                        as *const i8,
                    name,
                    0 as *const xmlChar,
                );
                if unsafe { xmlDictOwns_safe((safe_ctxt).dict, name) } == 0 {
                    unsafe { xmlFree_safe(name as *mut xmlChar as *mut ()) };
                }
                break;
            } else {
                tmp = unsafe { (*tmp).next }
            }
        }
        if tmp.is_null() {
            cur = unsafe { xmlCreateEnumeration_safe(name) };
            if cur.is_null() {
                unsafe { xmlFreeEnumeration_safe(ret) };
                return 0 as xmlEnumerationPtr;
            }
            if last.is_null() {
                last = cur;
                ret = last
            } else {
                unsafe {
                    (*last).next = cur;
                }
                last = cur
            }
        }
        xmlSkipBlankChars(ctxt);
        if unsafe { !(*(*(*ctxt).input).cur == '|' as u8) } {
            break;
        }
    }
    if unsafe { *(*(*ctxt).input).cur != ')' as u8 } {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_NOTATION_NOT_FINISHED, 0 as *const i8);
        }
        unsafe { xmlFreeEnumeration_safe(ret) };
        return 0 as xmlEnumerationPtr;
    }
    unsafe { xmlNextChar_safe(ctxt) };
    return ret;
}
/* *
* xmlParseEnumerationType:
* @ctxt:  an XML parser context
*
* parse an Enumeration attribute type.
*
* [59] Enumeration ::= '(' S? Nmtoken (S? '|' S? Nmtoken)* S? ')'
*
* [ VC: Enumeration ]
* Values of this type must match one of the Nmtoken tokens in
* the declaration
*
* Returns: the enumeration attribute tree built while parsing
*/

pub fn xmlParseEnumerationType(ctxt: xmlParserCtxtPtr) -> xmlEnumerationPtr {
    let mut name: *mut xmlChar;
    let mut ret: xmlEnumerationPtr = 0 as xmlEnumerationPtr;
    let mut last: xmlEnumerationPtr = 0 as xmlEnumerationPtr;
    let mut cur: xmlEnumerationPtr;
    let mut tmp: xmlEnumerationPtr;
    let safe_ctxt = unsafe { &mut *ctxt };
    unsafe {
        if *(*(*ctxt).input).cur != '(' as u8 {
            xmlFatalErr(ctxt, XML_ERR_ATTLIST_NOT_STARTED, 0 as *const i8);
            return 0 as xmlEnumerationPtr;
        }
    }
    SHRINK(ctxt);
    loop {
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        name = xmlParseNmtoken(ctxt);
        if name.is_null() {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_NMTOKEN_REQUIRED, 0 as *const i8);
            }
            return ret;
        }
        tmp = ret;
        while !tmp.is_null() {
            if unsafe { xmlStrEqual_safe(name, unsafe { (*tmp).name }) } != 0 {
                xmlValidityError(
                    ctxt,
                    XML_DTD_DUP_TOKEN,
                    b"standalone: attribute enumeration value token %s duplicated\n\x00"
                        as *const u8 as *const i8,
                    name,
                    0 as *const xmlChar,
                );
                if unsafe { xmlDictOwns_safe((safe_ctxt).dict, name) } == 0 {
                    unsafe { xmlFree_safe(name as *mut ()) };
                }
                break;
            } else {
                tmp = unsafe { (*tmp).next };
            }
        }
        if tmp.is_null() {
            cur = unsafe { xmlCreateEnumeration_safe(name) };
            if unsafe { xmlDictOwns_safe((safe_ctxt).dict, name) } == 0 {
                unsafe { xmlFree_safe(name as *mut ()) };
            }
            if cur.is_null() {
                unsafe { xmlFreeEnumeration_safe(ret) };
                return 0 as xmlEnumerationPtr;
            }
            if last.is_null() {
                last = cur;
                ret = last
            } else {
                unsafe {
                    (*last).next = cur;
                }
                last = cur
            }
        }
        xmlSkipBlankChars(ctxt);
        if unsafe { !(*(*(*ctxt).input).cur == '|' as u8) } {
            break;
        }
    }
    unsafe {
        if *(*(*ctxt).input).cur != ')' as u8 {
            xmlFatalErr(ctxt, XML_ERR_ATTLIST_NOT_FINISHED, 0 as *const i8);
            return ret;
        }
    }
    unsafe { xmlNextChar_safe(ctxt) };
    return ret;
}
/* *
* xmlParseEnumeratedType:
* @ctxt:  an XML parser context
* @tree:  the enumeration tree built while parsing
*
* parse an Enumerated attribute type.
*
* [57] EnumeratedType ::= NotationType | Enumeration
*
* [58] NotationType ::= 'NOTATION' S '(' S? Name (S? '|' S? Name)* S? ')'
*
*
* Returns: XML_ATTRIBUTE_ENUMERATION or XML_ATTRIBUTE_NOTATION
*/

pub fn xmlParseEnumeratedType(ctxt: xmlParserCtxtPtr, tree: *mut xmlEnumerationPtr) -> i32 {
    unsafe {
        if CMP8((*(*ctxt).input).cur, 'N', 'O', 'T', 'A', 'T', 'I', 'O', 'N') {
            SKIP(ctxt, 8);
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'NOTATION\'\n\x00" as *const u8 as *const i8,
                );
                return 0;
            }
            *tree = xmlParseNotationType(ctxt);
            if (*tree).is_null() {
                return 0;
            }
            return XML_ATTRIBUTE_NOTATION as i32;
        }
        *tree = xmlParseEnumerationType(ctxt);
        if (*tree).is_null() {
            return 0;
        }
    }
    return XML_ATTRIBUTE_ENUMERATION as i32;
}
/* *
* xmlParseAttributeType:
* @ctxt:  an XML parser context
* @tree:  the enumeration tree built while parsing
*
* parse the Attribute list def for an element
*
* [54] AttType ::= StringType | TokenizedType | EnumeratedType
*
* [55] StringType ::= 'CDATA'
*
* [56] TokenizedType ::= 'ID' | 'IDREF' | 'IDREFS' | 'ENTITY' |
*                        'ENTITIES' | 'NMTOKEN' | 'NMTOKENS'
*
* Validity constraints for attribute values syntax are checked in
* xmlValidateAttributeValue()
*
* [ VC: ID ]
* Values of type ID must match the Name production. A name must not
* appear more than once in an XML document as a value of this type;
* i.e., ID values must uniquely identify the elements which bear them.
*
* [ VC: One ID per Element Type ]
* No element type may have more than one ID attribute specified.
*
* [ VC: ID Attribute Default ]
* An ID attribute must have a declared default of #IMPLIED or #REQUIRED.
*
* [ VC: IDREF ]
* Values of type IDREF must match the Name production, and values
* of type IDREFS must match Names; each IDREF Name must match the value
* of an ID attribute on some element in the XML document; i.e. IDREF
* values must match the value of some ID attribute.
*
* [ VC: Entity Name ]
* Values of type ENTITY must match the Name production, values
* of type ENTITIES must match Names; each Entity Name must match the
* name of an unparsed entity declared in the DTD.
*
* [ VC: Name Token ]
* Values of type NMTOKEN must match the Nmtoken production; values
* of type NMTOKENS must match Nmtokens.
*
* Returns the attribute type
*/

pub fn xmlParseAttributeType(ctxt: xmlParserCtxtPtr, tree: *mut xmlEnumerationPtr) -> i32 {
    //@todo 削减unsafe范围
    unsafe {
        SHRINK(ctxt);
        if CMP5((*(*ctxt).input).cur, 'C', 'D', 'A', 'T', 'A') {
            SKIP(ctxt, 5);
            return XML_ATTRIBUTE_CDATA as i32;
        } else if CMP6((*(*ctxt).input).cur, 'I', 'D', 'R', 'E', 'F', 'S') {
            SKIP(ctxt, 6);
            return XML_ATTRIBUTE_IDREFS as i32;
        } else if CMP5((*(*ctxt).input).cur, 'I', 'D', 'R', 'E', 'F') {
            SKIP(ctxt, 5);
            return XML_ATTRIBUTE_IDREF as i32;
        } else if *(*(*ctxt).input).cur == 'I' as u8 && *(*(*ctxt).input).cur.offset(1) == 'D' as u8
        {
            SKIP(ctxt, 2);
            return XML_ATTRIBUTE_ID as i32;
        } else if CMP6((*(*ctxt).input).cur, 'E', 'N', 'T', 'I', 'T', 'Y') {
            SKIP(ctxt, 6);
            return XML_ATTRIBUTE_ENTITY as i32;
        } else if CMP8((*(*ctxt).input).cur, 'E', 'N', 'T', 'I', 'T', 'I', 'E', 'S') {
            SKIP(ctxt, 8);
            return XML_ATTRIBUTE_ENTITIES as i32;
        } else if CMP8((*(*ctxt).input).cur, 'N', 'M', 'T', 'O', 'K', 'E', 'N', 'S') {
            SKIP(ctxt, 8);
            return XML_ATTRIBUTE_NMTOKENS as i32;
        } else if CMP7((*(*ctxt).input).cur, 'N', 'M', 'T', 'O', 'K', 'E', 'N') {
            SKIP(ctxt, 7);
            return XML_ATTRIBUTE_NMTOKEN as i32;
        }
    }
    return xmlParseEnumeratedType(ctxt, tree);
}
/* *
* xmlParseAttributeListDecl:
* @ctxt:  an XML parser context
*
* : parse the Attribute list def for an element
*
* [52] AttlistDecl ::= '<!ATTLIST' S Name AttDef* S? '>'
*
* [53] AttDef ::= S Name S AttType S DefaultDecl
*
*/

pub fn xmlParseAttributeListDecl(ctxt: xmlParserCtxtPtr) {
    let mut elemName: *const xmlChar;
    let mut attrName: *const xmlChar;
    let mut tree: xmlEnumerationPtr;
    //@todo 削减unsafe范围
    unsafe {
        if CMP9(
            (*(*ctxt).input).cur,
            '<',
            '!',
            'A',
            'T',
            'T',
            'L',
            'I',
            'S',
            'T',
        ) {
            let inputid: i32 = (*(*ctxt).input).id;
            SKIP(ctxt, 9);
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'<!ATTLIST\'\n\x00" as *const u8 as *const i8,
                );
            }
            elemName = xmlParseName(ctxt);
            if elemName.is_null() {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_NAME_REQUIRED,
                    b"ATTLIST: no name for Element\n\x00" as *const u8 as *const i8,
                );
                return;
            }
            xmlSkipBlankChars(ctxt);
            GROW(ctxt);
            loop {
                if (!(*(*(*ctxt).input).cur != '>' as u8
                    && (*ctxt).instate != XML_PARSER_EOF as i32))
                {
                    break;
                }
                let mut type_0: i32;
                let mut def: i32;
                let mut defaultValue: *mut xmlChar = 0 as *mut xmlChar;
                GROW(ctxt);
                tree = 0 as xmlEnumerationPtr;
                attrName = xmlParseName(ctxt);
                if attrName.is_null() {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_NAME_REQUIRED,
                        b"ATTLIST: no name for Attribute\n\x00" as *const u8 as *const i8,
                    );
                    break;
                }
                GROW(ctxt);
                if xmlSkipBlankChars(ctxt) == 0 {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_SPACE_REQUIRED,
                        b"Space required after the attribute name\n\x00" as *const u8 as *const i8,
                    );
                    break;
                }
                type_0 = xmlParseAttributeType(ctxt, &mut tree);
                if type_0 <= 0 {
                    break;
                }
                GROW(ctxt);
                if xmlSkipBlankChars(ctxt) == 0 {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_SPACE_REQUIRED,
                        b"Space required after the attribute type\n\x00" as *const u8 as *const i8,
                    );
                    if !tree.is_null() {
                        xmlFreeEnumeration_safe(tree);
                    }
                    break;
                }
                def = xmlParseDefaultDecl(ctxt, &mut defaultValue);
                if def <= 0 {
                    if !defaultValue.is_null() {
                        xmlFree_safe(defaultValue as *mut ());
                    }
                    if !tree.is_null() {
                        xmlFreeEnumeration_safe(tree);
                    }
                    break;
                }
                if type_0 != XML_ATTRIBUTE_CDATA as i32 && !defaultValue.is_null() {
                    xmlAttrNormalizeSpace(defaultValue, defaultValue);
                }
                GROW(ctxt);
                if *(*(*ctxt).input).cur != '>' as u8 {
                    if xmlSkipBlankChars(ctxt) == 0 {
                        xmlFatalErrMsg(
                            ctxt,
                            XML_ERR_SPACE_REQUIRED,
                            b"Space required after the attribute default value\n\x00" as *const u8
                                as *const i8,
                        );
                        if !defaultValue.is_null() {
                            xmlFree_safe(defaultValue as *mut ());
                        }
                        if !tree.is_null() {
                            xmlFreeEnumeration_safe(tree);
                        }
                        break;
                    }
                }
                if !(*ctxt).sax.is_null()
                    && (*ctxt).disableSAX == 0
                    && (*(*ctxt).sax).attributeDecl.is_some()
                {
                    (*(*ctxt).sax)
                        .attributeDecl
                        .expect("non-null function pointer")(
                        (*ctxt).userData,
                        elemName,
                        attrName,
                        type_0,
                        def,
                        defaultValue,
                        tree,
                    );
                } else if !tree.is_null() {
                    xmlFreeEnumeration_safe(tree);
                }
                if (*ctxt).sax2 != 0
                    && !defaultValue.is_null()
                    && def != XML_ATTRIBUTE_IMPLIED as i32
                    && def != XML_ATTRIBUTE_REQUIRED as i32
                {
                    xmlAddDefAttrs(ctxt, elemName, attrName, defaultValue);
                }
                if (*ctxt).sax2 != 0 {
                    xmlAddSpecialAttr(ctxt, elemName, attrName, type_0);
                }
                if !defaultValue.is_null() {
                    xmlFree_safe(defaultValue as *mut ());
                }
                GROW(ctxt);
            }
            if *(*(*ctxt).input).cur == '>' as u8 {
                if inputid != (*(*ctxt).input).id {
                    xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                   b"Attribute list declaration doesn\'t start and stop in the same entity\n\x00"
                                   as *const u8 as *const i8);
                }
                xmlNextChar_safe(ctxt);
            }
        };
    }
}
/* *
* xmlParseElementMixedContentDecl:
* @ctxt:  an XML parser context
* @inputchk:  the input used for the current entity, needed for boundary checks
*
* parse the declaration for a Mixed Element content
* The leading '(' and spaces have been skipped in xmlParseElementContentDecl
*
* [51] Mixed ::= '(' S? '#PCDATA' (S? '|' S? Name)* S? ')*' |
*                '(' S? '#PCDATA' S? ')'
*
* [ VC: Proper Group/PE Nesting ] applies to [51] too (see [49])
*
* [ VC: No Duplicate Types ]
* The same name must not appear more than once in a single
* mixed-content declaration.
*
* returns: the list of the xmlElementContentPtr describing the element choices
*/

pub fn xmlParseElementMixedContentDecl(
    ctxt: xmlParserCtxtPtr,
    inputchk: i32,
) -> xmlElementContentPtr {
    let mut ret: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut cur: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut n: xmlElementContentPtr = 0 as *mut xmlElementContent;
    let mut elem: *const xmlChar = 0 as *const xmlChar;
    unsafe {
        GROW(ctxt);
        if CMP7((*(*ctxt).input).cur, '#', 'P', 'C', 'D', 'A', 'T', 'A') {
            SKIP(ctxt, 7);
            xmlSkipBlankChars(ctxt);
            SHRINK(ctxt);
            if *(*(*ctxt).input).cur == ')' as u8 {
                if (*(*ctxt).input).id != inputchk {
                    xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                   b"Element content declaration doesn\'t start and stop in the same entity\n\x00"
                                   as *const u8 as *const i8);
                }
                xmlNextChar_safe(ctxt);
                ret = xmlNewDocElementContent_safe(
                    (*ctxt).myDoc,
                    0 as *const xmlChar,
                    XML_ELEMENT_CONTENT_PCDATA,
                );
                if ret.is_null() {
                    return 0 as xmlElementContentPtr;
                }
                if *(*(*ctxt).input).cur == '*' as u8 {
                    (*ret).ocur = XML_ELEMENT_CONTENT_MULT;
                    xmlNextChar_safe(ctxt);
                }
                return ret;
            }
            if *(*(*ctxt).input).cur == '(' as u8 || *(*(*ctxt).input).cur == '|' as u8 {
                cur = xmlNewDocElementContent_safe(
                    (*ctxt).myDoc,
                    0 as *const xmlChar,
                    XML_ELEMENT_CONTENT_PCDATA,
                );
                ret = cur;
                if ret.is_null() {
                    return 0 as xmlElementContentPtr;
                }
            }
            loop {
                if (!(*(*(*ctxt).input).cur == '|' as u8 && (*ctxt).instate != XML_PARSER_EOF)) {
                    break;
                }
                xmlNextChar_safe(ctxt);
                if elem.is_null() {
                    ret = xmlNewDocElementContent_safe(
                        (*ctxt).myDoc,
                        0 as *const xmlChar,
                        XML_ELEMENT_CONTENT_OR,
                    );
                    if ret.is_null() {
                        xmlFreeDocElementContent_safe((*ctxt).myDoc, cur);
                        return 0 as xmlElementContentPtr;
                    }
                    (*ret).c1 = cur;
                    if !cur.is_null() {
                        (*cur).parent = ret
                    }
                    cur = ret
                } else {
                    n = xmlNewDocElementContent_safe(
                        (*ctxt).myDoc,
                        0 as *const xmlChar,
                        XML_ELEMENT_CONTENT_OR,
                    );
                    if n.is_null() {
                        xmlFreeDocElementContent_safe((*ctxt).myDoc, ret);
                        return 0 as xmlElementContentPtr;
                    }
                    (*n).c1 = xmlNewDocElementContent_safe(
                        (*ctxt).myDoc,
                        elem,
                        XML_ELEMENT_CONTENT_ELEMENT,
                    );
                    if !(*n).c1.is_null() {
                        (*(*n).c1).parent = n
                    }
                    (*cur).c2 = n;
                    if !n.is_null() {
                        (*n).parent = cur
                    }
                    cur = n
                }
                xmlSkipBlankChars(ctxt);
                elem = xmlParseName(ctxt);
                if elem.is_null() {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_NAME_REQUIRED,
                        b"xmlParseElementMixedContentDecl : Name expected\n\x00" as *const u8
                            as *const i8,
                    );
                    xmlFreeDocElementContent_safe((*ctxt).myDoc, ret);
                    return 0 as xmlElementContentPtr;
                }
                xmlSkipBlankChars(ctxt);
                GROW(ctxt);
            }
            if *(*(*ctxt).input).cur == ')' as u8 && *(*(*ctxt).input).cur.offset(1) == '*' as u8 {
                if !elem.is_null() {
                    (*cur).c2 = xmlNewDocElementContent_safe(
                        (*ctxt).myDoc,
                        elem,
                        XML_ELEMENT_CONTENT_ELEMENT,
                    );
                    if !(*cur).c2.is_null() {
                        (*(*cur).c2).parent = cur
                    }
                }
                if !ret.is_null() {
                    (*ret).ocur = XML_ELEMENT_CONTENT_MULT
                }
                if (*(*ctxt).input).id != inputchk {
                    xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                   b"Element content declaration doesn\'t start and stop in the same entity\n\x00"
                                   as *const u8 as *const i8);
                }
                SKIP(ctxt, 2);
            } else {
                xmlFreeDocElementContent_safe((*ctxt).myDoc, ret);
                xmlFatalErr(ctxt, XML_ERR_MIXED_NOT_STARTED, 0 as *const i8);
                return 0 as xmlElementContentPtr;
            }
        } else {
            xmlFatalErr(ctxt, XML_ERR_PCDATA_REQUIRED, 0 as *const i8);
        }
    }
    return ret;
}
/* *
* xmlParseElementChildrenContentDeclPriv:
* @ctxt:  an XML parser context
* @inputchk:  the input used for the current entity, needed for boundary checks
* @depth: the level of recursion
*
* parse the declaration for a Mixed Element content
* The leading '(' and spaces have been skipped in xmlParseElementContentDecl
*
*
* [47] children ::= (choice | seq) ('?' | '*' | '+')?
*
* [48] cp ::= (Name | choice | seq) ('?' | '*' | '+')?
*
* [49] choice ::= '(' S? cp ( S? '|' S? cp )* S? ')'
*
* [50] seq ::= '(' S? cp ( S? ',' S? cp )* S? ')'
*
* [ VC: Proper Group/PE Nesting ] applies to [49] and [50]
* TODO Parameter-entity replacement text must be properly nested
*	with parenthesized groups. That is to say, if either of the
*	opening or closing parentheses in a choice, seq, or Mixed
*	construct is contained in the replacement text for a parameter
*	entity, both must be contained in the same replacement text. For
*	interoperability, if a parameter-entity reference appears in a
*	choice, seq, or Mixed construct, its replacement text should not
*	be empty, and neither the first nor last non-blank character of
*	the replacement text should be a connector (| or ,).
*
* Returns the tree of xmlElementContentPtr describing the element
*          hierarchy.
*/
fn xmlParseElementChildrenContentDeclPriv(
    ctxt: xmlParserCtxtPtr,
    inputchk: i32,
    depth: i32,
) -> xmlElementContentPtr {
    let mut ret: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut cur: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut last: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut op: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let mut elem: *const xmlChar;
    let mut type_0: xmlChar = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    if depth > 128 && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 || depth > 2048 {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_ELEMCONTENT_NOT_FINISHED,
                b"xmlParseElementChildrenContentDecl : depth %d too deep, use XML_PARSE_HUGE\n\x00"
                    as *const u8 as *const i8,
                depth,
            );
        }
        return 0 as xmlElementContentPtr;
    }
    xmlSkipBlankChars(ctxt);
    GROW(ctxt);
    if unsafe { *(*(*ctxt).input).cur == '(' as u8 } {
        let inputid: i32 = unsafe { (*(*ctxt).input).id };
        /* Recurse on first child */
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        ret = xmlParseElementChildrenContentDeclPriv(ctxt, inputid, depth + 1);
        cur = ret;
        if cur.is_null() {
            return 0 as xmlElementContentPtr;
        }
        xmlSkipBlankChars(ctxt);
        GROW(ctxt);
    } else {
        elem = xmlParseName(ctxt);
        if elem.is_null() {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_ELEMCONTENT_NOT_STARTED, 0 as *const i8);
            }
            return 0 as xmlElementContentPtr;
        }
        ret = unsafe {
            xmlNewDocElementContent_safe((safe_ctxt).myDoc, elem, XML_ELEMENT_CONTENT_ELEMENT)
        };
        cur = ret;
        if cur.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return 0 as xmlElementContentPtr;
        }
        GROW(ctxt);
        unsafe {
            if *(*(*ctxt).input).cur == '?' as u8 {
                (*cur).ocur = XML_ELEMENT_CONTENT_OPT;
                xmlNextChar_safe(ctxt);
            } else if *(*(*ctxt).input).cur == '*' as u8 {
                (*cur).ocur = XML_ELEMENT_CONTENT_MULT;
                xmlNextChar_safe(ctxt);
            } else if *(*(*ctxt).input).cur == '+' as u8 {
                (*cur).ocur = XML_ELEMENT_CONTENT_PLUS;
                xmlNextChar_safe(ctxt);
            } else {
                (*cur).ocur = XML_ELEMENT_CONTENT_ONCE
            }
        }
        GROW(ctxt);
    }
    xmlSkipBlankChars(ctxt);
    SHRINK(ctxt);
    loop {
        if (!(unsafe { *(*(*ctxt).input).cur != ')' as u8 && (*ctxt).instate != XML_PARSER_EOF })) {
            break;
        }
        /*
         * Each loop we parse one separator and one element.
         */
        if unsafe { *(*(*ctxt).input).cur == ',' as u8 } {
            if type_0 == 0 {
                unsafe { type_0 = *(*(*ctxt).input).cur };
            } else if unsafe { type_0 != *(*(*ctxt).input).cur } {
                xmlFatalErrMsgInt(
                    ctxt,
                    XML_ERR_SEPARATOR_REQUIRED,
                    b"xmlParseElementChildrenContentDecl : \'%c\' expected\n\x00" as *const u8
                        as *const i8,
                    type_0 as i32,
                );
                if !last.is_null() && last != ret {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, last) };
                }
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            unsafe { xmlNextChar_safe(ctxt) };
            op = unsafe {
                xmlNewDocElementContent_safe(
                    (safe_ctxt).myDoc,
                    0 as *const xmlChar,
                    XML_ELEMENT_CONTENT_SEQ,
                )
            };
            if op.is_null() {
                if !last.is_null() && last != ret {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, last) };
                }
                unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                return 0 as xmlElementContentPtr;
            }
            if last.is_null() {
                unsafe {
                    (*op).c1 = ret;
                    if !ret.is_null() {
                        (*ret).parent = op
                    }
                }
                cur = op;
                ret = cur
            } else {
                unsafe {
                    (*cur).c2 = op;
                    if !op.is_null() {
                        (*op).parent = cur
                    }
                    (*op).c1 = last;
                    if !last.is_null() {
                        (*last).parent = op
                    }
                }
                cur = op;
                last = 0 as xmlElementContentPtr
            }
        } else if unsafe { *(*(*ctxt).input).cur == '|' as u8 } {
            if type_0 == 0 {
                type_0 = unsafe { *(*(*ctxt).input).cur }
            } else if unsafe { type_0 != *(*(*ctxt).input).cur } {
                xmlFatalErrMsgInt(
                    ctxt,
                    XML_ERR_SEPARATOR_REQUIRED,
                    b"xmlParseElementChildrenContentDecl : \'%c\' expected\n\x00" as *const u8
                        as *const i8,
                    type_0 as i32,
                );
                if !last.is_null() && last != ret {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, last) };
                }
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            unsafe { xmlNextChar_safe(ctxt) };
            op = unsafe {
                xmlNewDocElementContent_safe(
                    (safe_ctxt).myDoc,
                    0 as *const xmlChar,
                    XML_ELEMENT_CONTENT_OR,
                )
            };
            if op.is_null() {
                if !last.is_null() && last != ret {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, last) };
                }
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            if last.is_null() {
                unsafe {
                    (*op).c1 = ret;
                    if !ret.is_null() {
                        (*ret).parent = op
                    }
                }
                cur = op;
                ret = cur
            } else {
                unsafe {
                    (*cur).c2 = op;
                    if !op.is_null() {
                        (*op).parent = cur
                    }
                    (*op).c1 = last;
                    if !last.is_null() {
                        (*last).parent = op
                    }
                }
                cur = op;
                last = 0 as xmlElementContentPtr
            }
        } else {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_ELEMCONTENT_NOT_FINISHED, 0 as *const i8);
            }
            if !last.is_null() && last != ret {
                unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, last) };
            }
            if !ret.is_null() {
                unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
            }
            return 0 as xmlElementContentPtr;
        }
        GROW(ctxt);
        xmlSkipBlankChars(ctxt);
        GROW(ctxt);
        if unsafe { *(*(*ctxt).input).cur == '(' as u8 } {
            let inputid_0: i32 = unsafe { (*(*ctxt).input).id };
            /*
             * Detect "Name | Name , Name" error
             */
            /*
             * Detect "Name , Name | Name" error
             */
            /* Recurse on second child */
            unsafe { xmlNextChar_safe(ctxt) };
            xmlSkipBlankChars(ctxt);
            last = xmlParseElementChildrenContentDeclPriv(ctxt, inputid_0, depth + 1);
            if last.is_null() {
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            xmlSkipBlankChars(ctxt);
        } else {
            elem = xmlParseName(ctxt);
            if elem.is_null() {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_ELEMCONTENT_NOT_STARTED, 0 as *const i8);
                }
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            last = unsafe {
                xmlNewDocElementContent_safe((safe_ctxt).myDoc, elem, XML_ELEMENT_CONTENT_ELEMENT)
            };
            if last.is_null() {
                if !ret.is_null() {
                    unsafe { xmlFreeDocElementContent_safe((safe_ctxt).myDoc, ret) };
                }
                return 0 as xmlElementContentPtr;
            }
            unsafe {
                if *(*(*ctxt).input).cur == '?' as u8 {
                    (*last).ocur = XML_ELEMENT_CONTENT_OPT;
                    xmlNextChar_safe(ctxt);
                } else if *(*(*ctxt).input).cur == '*' as u8 {
                    (*last).ocur = XML_ELEMENT_CONTENT_MULT;
                    xmlNextChar_safe(ctxt);
                } else if *(*(*ctxt).input).cur == '+' as u8 {
                    (*last).ocur = XML_ELEMENT_CONTENT_PLUS;
                    xmlNextChar_safe(ctxt);
                } else {
                    (*last).ocur = XML_ELEMENT_CONTENT_ONCE
                }
            }
        }
        xmlSkipBlankChars(ctxt);
        GROW(ctxt);
    }
    if !cur.is_null() && !last.is_null() {
        unsafe {
            (*cur).c2 = last;
            if !last.is_null() {
                (*last).parent = cur
            }
        }
    }
    unsafe {
        if (*(*ctxt).input).id != inputchk {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_ENTITY_BOUNDARY,
                b"Element content declaration doesn\'t start and stop in the same entity\n\x00"
                    as *const u8 as *const i8,
            );
        }
    }
    unsafe { xmlNextChar_safe(ctxt) };
    unsafe {
        if *(*(*ctxt).input).cur == '?' as u8 {
            if !ret.is_null() {
                if (*ret).ocur as u32 == XML_ELEMENT_CONTENT_PLUS as u32
                    || (*ret).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32
                {
                    (*ret).ocur = XML_ELEMENT_CONTENT_MULT
                } else {
                    (*ret).ocur = XML_ELEMENT_CONTENT_OPT
                }
            }
            xmlNextChar_safe(ctxt);
        } else if *(*(*ctxt).input).cur == '*' as u8 {
            if !ret.is_null() {
                (*ret).ocur = XML_ELEMENT_CONTENT_MULT;
                cur = ret;
                /*
                 * Some normalization:
                 * (a | b* | c?)* == (a | b | c)*
                 */
                while !cur.is_null() && (*cur).type_0 as u32 == XML_ELEMENT_CONTENT_OR as u32 {
                    if !(*cur).c1.is_null()
                        && ((*(*cur).c1).ocur as u32 == XML_ELEMENT_CONTENT_OPT as u32
                            || (*(*cur).c1).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32)
                    {
                        (*(*cur).c1).ocur = XML_ELEMENT_CONTENT_ONCE
                    }
                    if !(*cur).c2.is_null()
                        && ((*(*cur).c2).ocur as u32 == XML_ELEMENT_CONTENT_OPT as u32
                            || (*(*cur).c2).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32)
                    {
                        (*(*cur).c2).ocur = XML_ELEMENT_CONTENT_ONCE
                    }
                    cur = (*cur).c2
                }
            }
            xmlNextChar_safe(ctxt);
        } else if *(*(*ctxt).input).cur == '+' as u8 {
            if !ret.is_null() {
                let mut found: i32 = 0;
                if (*ret).ocur as u32 == XML_ELEMENT_CONTENT_OPT as u32
                    || (*ret).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32
                {
                    (*ret).ocur = XML_ELEMENT_CONTENT_MULT
                } else {
                    (*ret).ocur = XML_ELEMENT_CONTENT_PLUS
                }
                /*
                 * Some normalization:
                 * (a | b*)+ == (a | b)*
                 * (a | b?)+ == (a | b)*
                 */
                while !cur.is_null() && (*cur).type_0 as u32 == XML_ELEMENT_CONTENT_OR as u32 {
                    if !(*cur).c1.is_null()
                        && ((*(*cur).c1).ocur as u32 == XML_ELEMENT_CONTENT_OPT as u32
                            || (*(*cur).c1).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32)
                    {
                        (*(*cur).c1).ocur = XML_ELEMENT_CONTENT_ONCE;
                        found = 1
                    }
                    if !(*cur).c2.is_null()
                        && ((*(*cur).c2).ocur as u32 == XML_ELEMENT_CONTENT_OPT as u32
                            || (*(*cur).c2).ocur as u32 == XML_ELEMENT_CONTENT_MULT as u32)
                    {
                        (*(*cur).c2).ocur = XML_ELEMENT_CONTENT_ONCE;
                        found = 1
                    }
                    cur = (*cur).c2
                }
                if found != 0 {
                    (*ret).ocur = XML_ELEMENT_CONTENT_MULT
                }
            }
            xmlNextChar_safe(ctxt);
        }
    }
    return ret;
}
/* *
* xmlParseElementChildrenContentDecl:
* @ctxt:  an XML parser context
* @inputchk:  the input used for the current entity, needed for boundary checks
*
* parse the declaration for a Mixed Element content
* The leading '(' and spaces have been skipped in xmlParseElementContentDecl
*
* [47] children ::= (choice | seq) ('?' | '*' | '+')?
*
* [48] cp ::= (Name | choice | seq) ('?' | '*' | '+')?
*
* [49] choice ::= '(' S? cp ( S? '|' S? cp )* S? ')'
*
* [50] seq ::= '(' S? cp ( S? ',' S? cp )* S? ')'
*
* [ VC: Proper Group/PE Nesting ] applies to [49] and [50]
* TODO Parameter-entity replacement text must be properly nested
*	with parenthesized groups. That is to say, if either of the
*	opening or closing parentheses in a choice, seq, or Mixed
*	construct is contained in the replacement text for a parameter
*	entity, both must be contained in the same replacement text. For
*	interoperability, if a parameter-entity reference appears in a
*	choice, seq, or Mixed construct, its replacement text should not
*	be empty, and neither the first nor last non-blank character of
*	the replacement text should be a connector (| or ,).
*
* Returns the tree of xmlElementContentPtr describing the element
*          hierarchy.
*/

pub fn xmlParseElementChildrenContentDecl(
    ctxt: xmlParserCtxtPtr,
    inputchk: i32,
) -> xmlElementContentPtr {
    /* stub left for API/ABI compat */
    return xmlParseElementChildrenContentDeclPriv(ctxt, inputchk, 1);
}
/* *
* xmlParseElementContentDecl:
* @ctxt:  an XML parser context
* @name:  the name of the element being defined.
* @result:  the Element Content pointer will be stored here if any
*
* parse the declaration for an Element content either Mixed or Children,
* the cases EMPTY and ANY are handled directly in xmlParseElementDecl
*
* [46] contentspec ::= 'EMPTY' | 'ANY' | Mixed | children
*
* returns: the type of element content XML_ELEMENT_TYPE_xxx
*/

pub fn xmlParseElementContentDecl(
    ctxt: xmlParserCtxtPtr,
    name: *const xmlChar,
    result: *mut xmlElementContentPtr,
) -> i32 {
    let mut tree: xmlElementContentPtr = 0 as xmlElementContentPtr;
    let safe_ctxt = unsafe { &mut *ctxt };
    let inputid: i32 = unsafe { (*(*ctxt).input).id };
    let mut res: i32 = 0;
    unsafe {
        *result = 0 as xmlElementContentPtr;
        if *(*(*ctxt).input).cur != '(' as u8 {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_ELEMCONTENT_NOT_STARTED,
                b"xmlParseElementContentDecl : %s \'(\' expected\n\x00" as *const u8 as *const i8,
                name,
            );
            return -(1);
        }
    }
    unsafe { xmlNextChar_safe(ctxt) };
    GROW(ctxt);
    if (safe_ctxt).instate == XML_PARSER_EOF {
        return -(1);
    }
    xmlSkipBlankChars(ctxt);
    if CMP7(
        unsafe { (*(*ctxt).input).cur },
        '#',
        'P',
        'C',
        'D',
        'A',
        'T',
        'A',
    ) {
        tree = xmlParseElementMixedContentDecl(ctxt, inputid);
        res = XML_ELEMENT_TYPE_MIXED as i32
    } else {
        tree = xmlParseElementChildrenContentDeclPriv(ctxt, inputid, 1);
        res = XML_ELEMENT_TYPE_ELEMENT as i32
    }
    xmlSkipBlankChars(ctxt);
    unsafe { *result = tree };
    return res;
}
/* *
* xmlParseElementDecl:
* @ctxt:  an XML parser context
*
* parse an Element declaration.
*
* [45] elementdecl ::= '<!ELEMENT' S Name S contentspec S? '>'
*
* [ VC: Unique Element Type Declaration ]
* No element type may be declared more than once
*
* Returns the type of the element, or -1 in case of error
*/

pub fn xmlParseElementDecl(ctxt: xmlParserCtxtPtr) -> i32 {
    let mut name: *const xmlChar;
    let mut ret: i32 = -(1);
    let mut content: xmlElementContentPtr = 0 as xmlElementContentPtr;
    /* GROW; done in the caller */
    unsafe {
        if CMP9(
            (*(*ctxt).input).cur,
            '<',
            '!',
            'E',
            'L',
            'E',
            'M',
            'E',
            'N',
            'T',
        ) {
            let inputid: i32 = (*(*ctxt).input).id;
            SKIP(ctxt, 9);
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after \'ELEMENT\'\n\x00" as *const u8 as *const i8,
                );
                return -(1);
            }
            name = xmlParseName(ctxt);
            if name.is_null() {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_NAME_REQUIRED,
                    b"xmlParseElementDecl: no name for Element\n\x00" as *const u8 as *const i8,
                );
                return -(1);
            }
            if xmlSkipBlankChars(ctxt) == 0 {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_SPACE_REQUIRED,
                    b"Space required after the element name\n\x00" as *const u8 as *const i8,
                );
            }
            if CMP5((*(*ctxt).input).cur, 'E', 'M', 'P', 'T', 'Y') {
                SKIP(ctxt, 5);
                /*
                 * Element must always be empty.
                 */
                ret = XML_ELEMENT_TYPE_EMPTY as i32
            } else if *(*(*ctxt).input).cur == 'A' as u8
                && *(*(*ctxt).input).cur.offset(1) == 'N' as u8
                && *(*(*ctxt).input).cur.offset(2) == 'Y' as u8
            {
                SKIP(ctxt, 3);
                /*
                 * Element is a generic container.
                 */
                ret = XML_ELEMENT_TYPE_ANY as i32
            } else if *(*(*ctxt).input).cur as i32 == '(' as i32 {
                ret = xmlParseElementContentDecl(ctxt, name, &mut content)
            } else {
                /*
                 * [ WFC: PEs in Internal Subset ] error handling.
                 */
                if *(*(*ctxt).input).cur == '%' as u8
                    && (*ctxt).external == 0
                    && (*ctxt).inputNr == 1
                {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_PEREF_IN_INT_SUBSET,
                        b"PEReference: forbidden within markup decl in internal subset\n\x00"
                            as *const u8 as *const i8,
                    );
                } else {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ELEMCONTENT_NOT_STARTED,
                        b"xmlParseElementDecl: \'EMPTY\', \'ANY\' or \'(\' expected\n\x00"
                            as *const u8 as *const i8,
                    );
                }
                return -(1);
            }
            xmlSkipBlankChars(ctxt);
            if *(*(*ctxt).input).cur != '>' as u8 {
                xmlFatalErr(ctxt, XML_ERR_GT_REQUIRED, 0 as *const i8);
                if !content.is_null() {
                    xmlFreeDocElementContent_safe((*ctxt).myDoc, content);
                }
            } else {
                if inputid != (*(*ctxt).input).id {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ENTITY_BOUNDARY,
                        b"Element declaration doesn\'t start and stop in the same entity\n\x00"
                            as *const u8 as *const i8,
                    );
                }
                xmlNextChar_safe(ctxt);
                if !(*ctxt).sax.is_null()
                    && (*ctxt).disableSAX == 0
                    && (*(*ctxt).sax).elementDecl.is_some()
                {
                    if !content.is_null() {
                        (*content).parent = 0 as *mut _xmlElementContent
                    }
                    (*(*ctxt).sax)
                        .elementDecl
                        .expect("non-null function pointer")(
                        (*ctxt).userData, name, ret, content
                    );
                    if !content.is_null() && (*content).parent.is_null() {
                        /*
                         * this is a trick: if xmlAddElementDecl is called,
                         * instead of copying the full tree it is plugged directly
                         * if called from the parser. Avoid duplicating the
                         * interfaces or change the API/ABI
                         */
                        xmlFreeDocElementContent_safe((*ctxt).myDoc, content);
                    }
                } else if !content.is_null() {
                    xmlFreeDocElementContent_safe((*ctxt).myDoc, content);
                }
            }
        }
    }
    return ret;
}
/* *
* xmlParseConditionalSections
* @ctxt:  an XML parser context
*
* [61] conditionalSect ::= includeSect | ignoreSect
* [62] includeSect ::= '<![' S? 'INCLUDE' S? '[' extSubsetDecl ']]>'
* [63] ignoreSect ::= '<![' S? 'IGNORE' S? '[' ignoreSectContents* ']]>'
* [64] ignoreSectContents ::= Ignore ('<![' ignoreSectContents ']]>' Ignore)*
* [65] Ignore ::= Char* - (Char* ('<![' | ']]>') Char*)
*/
fn xmlParseConditionalSections(ctxt: xmlParserCtxtPtr) {
    let mut inputIds: *mut i32 = 0 as *mut i32;
    let mut inputIdsSize: size_t = 0;
    let mut depth: size_t = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    loop {
        if (!((safe_ctxt).instate != XML_PARSER_EOF)) {
            break;
        }
        unsafe {
            if *(*(*ctxt).input).cur == '<' as u8
                && *(*(*ctxt).input).cur.offset(1) == '!' as u8
                && *(*(*ctxt).input).cur.offset(2) == '[' as u8
            {
                let id: i32 = (*(*ctxt).input).id;
                SKIP(ctxt, 3);
                xmlSkipBlankChars(ctxt);
                if CMP7((*(*ctxt).input).cur, 'I', 'N', 'C', 'L', 'U', 'D', 'E') {
                    SKIP(ctxt, 7);
                    xmlSkipBlankChars(ctxt);
                    if *(*(*ctxt).input).cur != '[' as u8 {
                        xmlFatalErr(ctxt, XML_ERR_CONDSEC_INVALID, 0 as *const i8);
                        xmlHaltParser(ctxt);
                        break;
                    } else {
                        if (*(*ctxt).input).id != id {
                            xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                           b"All markup of the conditional section is not in the same entity\n\x00"
                                           as *const u8 as
                                           *const i8);
                        }
                        xmlNextChar_safe(ctxt);
                        if inputIdsSize <= depth {
                            let mut tmp: *mut i32;
                            inputIdsSize = if inputIdsSize == 0 {
                                4
                            } else {
                                inputIdsSize * 2
                            };
                            tmp = xmlRealloc_safe(
                                inputIds as *mut (),
                                inputIdsSize.wrapping_mul(size_of::<i32>() as u64),
                            ) as *mut i32;
                            if tmp.is_null() {
                                xmlErrMemory(ctxt, 0 as *const i8);
                                break;
                            } else {
                                inputIds = tmp
                            }
                        }
                        *inputIds.offset(depth as isize) = id;
                        depth = depth.wrapping_add(1)
                    }
                } else if CMP6((*(*ctxt).input).cur, 'I', 'G', 'N', 'O', 'R', 'E') {
                    let mut state: i32 = 0;
                    let mut instate: xmlParserInputState = XML_PARSER_START;
                    let mut ignoreDepth: size_t = 0;
                    SKIP(ctxt, 6);
                    xmlSkipBlankChars(ctxt);
                    if *(*(*ctxt).input).cur != '[' as u8 {
                        xmlFatalErr(ctxt, XML_ERR_CONDSEC_INVALID, 0 as *const i8);
                        xmlHaltParser(ctxt);
                        break;
                    } else {
                        if (*(*ctxt).input).id != id {
                            xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                           b"All markup of the conditional section is not in the same entity\n\x00"
                                           as *const u8 as
                                           *const i8);
                        }
                        xmlNextChar_safe(ctxt);
                        /*
                         * Parse up to the end of the conditional section but disable
                         * SAX event generating DTD building in the meantime
                         */
                        state = (*ctxt).disableSAX;
                        instate = (*ctxt).instate;
                        if (*ctxt).recovery == 0 {
                            (*ctxt).disableSAX = 1
                        }
                        (*ctxt).instate = XML_PARSER_IGNORE;
                        loop {
                            if *(*(*ctxt).input).cur == 0 {
                                break;
                            }
                            if *(*(*ctxt).input).cur == '<' as u8
                                && *(*(*ctxt).input).cur.offset(1) == '!' as u8
                                && *(*(*ctxt).input).cur.offset(2) == '[' as u8
                            {
                                SKIP(ctxt, 3);
                                ignoreDepth += 1;
                                /* Check for integer overflow */
                                if !(ignoreDepth == 0) {
                                    continue;
                                }
                                xmlErrMemory(ctxt, 0 as *const i8);
                                break;
                            } else if *(*(*ctxt).input).cur == ']' as u8
                                && *(*(*ctxt).input).cur.offset(1) == ']' as u8
                                && *(*(*ctxt).input).cur.offset(2) == '>' as u8
                            {
                                if ignoreDepth == 0 {
                                    break;
                                }
                                SKIP(ctxt, 3);
                                ignoreDepth -= 1;
                            } else {
                                xmlNextChar_safe(ctxt);
                            }
                        }
                        (*ctxt).disableSAX = state;
                        (*ctxt).instate = instate;
                        if *(*(*ctxt).input).cur == 0 {
                            xmlFatalErr(ctxt, XML_ERR_CONDSEC_NOT_FINISHED, 0 as *const i8);
                            break;
                        } else {
                            if (*(*ctxt).input).id != id {
                                xmlFatalErrMsg(ctxt, XML_ERR_ENTITY_BOUNDARY,
                                               b"All markup of the conditional section is not in the same entity\n\x00"
                                               as *const u8 as
                                               *const i8);
                            }
                            SKIP(ctxt, 3);
                        }
                    }
                } else {
                    xmlFatalErr(ctxt, XML_ERR_CONDSEC_INVALID_KEYWORD, 0 as *const i8);
                    xmlHaltParser(ctxt);
                    break;
                }
            } else if depth > 0
                && *(*(*ctxt).input).cur == ']' as u8
                && *(*(*ctxt).input).cur.offset(1) == ']' as u8
                && *(*(*ctxt).input).cur.offset(2) == '>' as u8
            {
                depth -= 1;
                if (*(*ctxt).input).id != *inputIds.offset(depth as isize) {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ENTITY_BOUNDARY,
                        b"All markup of the conditional section is not in the same entity\n\x00"
                            as *const u8 as *const i8,
                    );
                }
                SKIP(ctxt, 3);
            } else {
                let check: *const xmlChar = (*(*ctxt).input).cur;
                let cons: u32 = (*(*ctxt).input).consumed as u32;
                xmlParseMarkupDecl(ctxt);
                if (*(*ctxt).input).cur == check && cons as u64 == (*(*ctxt).input).consumed {
                    xmlFatalErr(ctxt, XML_ERR_EXT_SUBSET_NOT_FINISHED, 0 as *const i8);
                    xmlHaltParser(ctxt);
                    break;
                }
            }
            if depth == 0 {
                break;
            }
            xmlSkipBlankChars(ctxt);
            GROW(ctxt);
        }
    }
    unsafe { xmlFree_safe(inputIds as *mut ()) };
}
/* *
* xmlParseMarkupDecl:
* @ctxt:  an XML parser context
*
* parse Markup declarations
*
* [29] markupdecl ::= elementdecl | AttlistDecl | EntityDecl |
*                     NotationDecl | PI | Comment
*
* [ VC: Proper Declaration/PE Nesting ]
* Parameter-entity replacement text must be properly nested with
* markup declarations. That is to say, if either the first character
* or the last character of a markup declaration (markupdecl above) is
* contained in the replacement text for a parameter-entity reference,
* both must be contained in the same replacement text.
*
* [ WFC: PEs in Internal Subset ]
* In the internal DTD subset, parameter-entity references can occur
* only where markup declarations can occur, not within markup declarations.
* (This does not apply to references that occur in external parameter
* entities or to the external subset.)
*/

pub fn xmlParseMarkupDecl(ctxt: xmlParserCtxtPtr) {
    let safe_ctxt = unsafe { &mut *ctxt };
    GROW(ctxt);
    unsafe {
        if *(*(*ctxt).input).cur == '<' as u8 {
            if *(*(*ctxt).input).cur.offset(1) == '!' as u8 {
                match *(*(*ctxt).input).cur.offset(2) as char {
                    'E' => {
                        if *(*(*ctxt).input).cur.offset(3) == 'L' as u8 {
                            xmlParseElementDecl(ctxt);
                        } else if *(*(*ctxt).input).cur.offset(3) == 'N' as u8 {
                            xmlParseEntityDecl(ctxt);
                        }
                    }
                    'A' => {
                        xmlParseAttributeListDecl(ctxt);
                    }
                    'N' => {
                        xmlParseNotationDecl(ctxt);
                    }
                    '-' => {
                        xmlParseComment(ctxt);
                    }
                    _ => {}
                }
            } else if *(*(*ctxt).input).cur.offset(1) == '?' as u8 {
                xmlParsePI(ctxt);
            }
        }
    }
    /*
     * detect requirement to exit there and act accordingly
     * and avoid having instate overridden later on
     */
    if (safe_ctxt).instate == XML_PARSER_EOF {
        return;
    }
    (safe_ctxt).instate = XML_PARSER_DTD;
}
/* *
* xmlParseTextDecl:
* @ctxt:  an XML parser context
*
* parse an XML declaration header for external entities
*
* [77] TextDecl ::= '<?xml' VersionInfo? EncodingDecl S? '?>'
*/

pub fn xmlParseTextDecl(ctxt: xmlParserCtxtPtr) {
    let mut version: *mut xmlChar = 0 as *mut xmlChar;
    let mut encoding: *const xmlChar = 0 as *const xmlChar;
    let mut oldstate: i32 = 0;
    let safe_ctxt = unsafe { &mut *ctxt };
    /*
     * We know that '<?xml' is here.
     */
    unsafe {
        if CMP5((*(*ctxt).input).cur, '<', '?', 'x', 'm', 'l')
            && IS_BLANK_CH((*(*ctxt).input).cur.offset(5))
        {
            SKIP(ctxt, 5);
        } else {
            xmlFatalErr(ctxt, XML_ERR_XMLDECL_NOT_STARTED, 0 as *const i8);
            return;
        }
    }
    /* Avoid expansion of parameter entities when skipping blanks. */
    oldstate = (safe_ctxt).instate as i32;
    (safe_ctxt).instate = XML_PARSER_START;
    if xmlSkipBlankChars(ctxt) == 0 {
        unsafe {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_SPACE_REQUIRED,
                b"Space needed after \'<?xml\'\n\x00" as *const u8 as *const i8,
            );
        }
    }
    /*
     * We may have the VersionInfo here.
     */
    unsafe {
        version = xmlParseVersionInfo(ctxt);
    }
    if version.is_null() {
        version = unsafe { xmlCharStrdup_safe(b"1.0\x00" as *const u8 as *const i8) }
    } else if xmlSkipBlankChars(ctxt) == 0 {
        unsafe {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_SPACE_REQUIRED,
                b"Space needed here\n\x00" as *const u8 as *const i8,
            );
        }
    }
    unsafe {
        (*(*ctxt).input).version = version;
    }
    /*
     * We must have the encoding declaration
     */
    unsafe {
        encoding = xmlParseEncodingDecl(ctxt);
    }
    if (safe_ctxt).errNo == XML_ERR_UNSUPPORTED_ENCODING as i32 {
        /*
         * The XML REC instructs us to stop parsing right here
         */
        (safe_ctxt).instate = oldstate as xmlParserInputState;
        return;
    }
    if encoding.is_null() && (safe_ctxt).errNo == XML_ERR_OK as i32 {
        unsafe {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_MISSING_ENCODING,
                b"Missing encoding in text declaration\n\x00" as *const u8 as *const i8,
            );
        }
    }
    xmlSkipBlankChars(ctxt);
    unsafe {
        if *(*(*ctxt).input).cur == '?' as u8 && *(*(*ctxt).input).cur.offset(1) == '>' as u8 {
            SKIP(ctxt, 2);
        } else if *(*(*ctxt).input).cur == '>' as u8 {
            /* Deprecated old WD ... */
            xmlFatalErr(ctxt, XML_ERR_XMLDECL_NOT_FINISHED, 0 as *const i8);
            xmlNextChar_safe(ctxt);
        } else {
            xmlFatalErr(ctxt, XML_ERR_XMLDECL_NOT_FINISHED, 0 as *const i8);
            while *(*(*ctxt).input).cur != 0 && *(*(*ctxt).input).cur != '>' as u8 {
                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(1)
            }
            xmlNextChar_safe(ctxt);
        }
    }
    (safe_ctxt).instate = oldstate as xmlParserInputState;
}

/* *
* xmlParseExternalSubset:
* @ctxt:  an XML parser context
* @ExternalID: the external identifier
* @SystemID: the system identifier (or URL)
*
* parse Markup declarations from an external subset
*
* [30] extSubset ::= textDecl? extSubsetDecl
*
* [31] extSubsetDecl ::= (markupdecl | conditionalSect | PEReference | S) *
*/
pub fn xmlParseExternalSubset(
    mut ctxt: xmlParserCtxtPtr,
    ExternalID: *const xmlChar,
    SystemID: *const xmlChar,
) {
    xmlDetectSAX2(ctxt);
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if safe_ctxt.progressive == 0
        && unsafe { ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    if safe_ctxt.encoding.is_null()
        && unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 >= 4 }
    {
        let mut start: [xmlChar; 4] = [0; 4];
        let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
        unsafe {
            start[0] = *(*safe_ctxt.input).cur;
            start[1] = *(*safe_ctxt.input).cur.offset(1);
            start[2] = *(*safe_ctxt.input).cur.offset(2);
            start[3] = *(*safe_ctxt.input).cur.offset(3);
        }
        enc = unsafe { xmlDetectCharEncoding_safe(start.as_mut_ptr(), 4) };
        if enc != XML_CHAR_ENCODING_NONE {
            unsafe { xmlSwitchEncoding_safe(ctxt, enc) };
        }
    }
    if unsafe {
        *((*safe_ctxt.input).cur as *mut u8).offset(0 as isize) == '<' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(1 as isize) == '?' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(2 as isize) == 'x' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(3 as isize) == 'm' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(4 as isize) == 'l' as u8
    } {
        xmlParseTextDecl(ctxt);
        if safe_ctxt.errNo == XML_ERR_UNSUPPORTED_ENCODING as i32 {
            /*
             * The XML REC instructs us to stop parsing right here
             */
            unsafe {
                xmlHaltParser(ctxt);
            }
            return;
        }
    }
    if safe_ctxt.myDoc.is_null() {
        safe_ctxt.myDoc =
            unsafe { xmlNewDoc_safe(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar) };
        if safe_ctxt.myDoc.is_null() {
            xmlErrMemory(ctxt, b"New Doc failed\x00" as *const u8 as *const i8);

            return;
        }
        unsafe { (*safe_ctxt.myDoc).properties = XML_DOC_INTERNAL as i32 }
    }
    if !safe_ctxt.myDoc.is_null() && unsafe { (*safe_ctxt.myDoc).intSubset.is_null() } {
        unsafe {
            xmlCreateIntSubset_safe(safe_ctxt.myDoc, 0 as *const xmlChar, ExternalID, SystemID)
        };
    }
    safe_ctxt.instate = XML_PARSER_DTD;
    safe_ctxt.external = 1;
    xmlSkipBlankChars(ctxt);

    while unsafe {
        *(*safe_ctxt.input).cur == '<' as u8
            && *(*safe_ctxt.input).cur.offset(1 as isize) == '?' as u8
            || *(*safe_ctxt.input).cur == '<' as u8
                && *(*safe_ctxt.input).cur.offset(1 as isize) == '!' as u8
            || *(*safe_ctxt.input).cur == '%' as u8
    } {
        let safe_input = unsafe { &mut *safe_ctxt.input };
        let mut check: *const xmlChar = safe_input.cur;
        let mut cons: u32 = safe_input.consumed as u32;
        if safe_ctxt.progressive == 0
            && unsafe { safe_input.end.offset_from(safe_input.cur) as i64 } < 250
        {
            xmlGROW(ctxt);
        }
        if unsafe {
            *safe_input.cur == '<' as u8
                && *safe_input.cur.offset(1 as isize) == '!' as u8
                && *safe_input.cur.offset(2 as isize) == '[' as u8
        } {
            xmlParseConditionalSections(ctxt);
        } else {
            xmlParseMarkupDecl(ctxt);
        }
        xmlSkipBlankChars(ctxt);
        if !(safe_input.cur == check && cons as u64 == safe_input.consumed) {
            continue;
        }
        unsafe { xmlFatalErr(ctxt, XML_ERR_EXT_SUBSET_NOT_FINISHED, 0 as *const i8) };
        break;
    }
    if unsafe { *(*safe_ctxt.input).cur as i32 } != 0 {
        unsafe { xmlFatalErr(ctxt, XML_ERR_EXT_SUBSET_NOT_FINISHED, 0 as *const i8) };
    };
}
/* *
* xmlParseReference:
* @ctxt:  an XML parser context
*
* parse and handle entity references in content, depending on the SAX
* interface, this may end-up in a call to character() if this is a
* CharRef, a predefined entity, if there is no reference() callback.
* or if the parser was asked to switch to that mode.
*
* [67] Reference ::= EntityRef | CharRef
*/

pub fn xmlParseReference(mut ctxt: xmlParserCtxtPtr) {
    let mut ent: xmlEntityPtr = 0 as *mut xmlEntity;
    let val: *mut xmlChar;
    let mut was_checked: i32 = 0;
    let mut list: xmlNodePtr = 0 as xmlNodePtr;
    let mut ret: xmlParserErrors = XML_ERR_OK;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*safe_ctxt.input).cur != '&' as u8 } {
        return;
    }
    /*
     * Simple case of a CharRef
     */
    if unsafe { *(*safe_ctxt.input).cur.offset(1 as isize) == '#' as u8 } {
        let mut i: i32 = 0;
        let mut out: [xmlChar; 16] = [0; 16];
        let hex: u8 = unsafe { *(*safe_ctxt.input).cur.offset(2 as isize) };
        let value: i32 = xmlParseCharRef(ctxt);
        if value == 0 {
            return;
        }
        if safe_ctxt.charset != XML_CHAR_ENCODING_UTF8 as i32 {
            /*
             * So we are using non-UTF-8 buffers
             * Check that the char fit on 8bits, if not
             * generate a CharRef.
             */
            if value <= 0xff {
                out[0] = value as xmlChar;
                out[1] = 0;

                if !safe_ctxt.sax.is_null()
                    && unsafe { (*safe_ctxt.sax).characters.is_some() }
                    && safe_ctxt.disableSAX == 0
                {
                    unsafe {
                        (*safe_ctxt.sax)
                            .characters
                            .expect("non-null function pointer")(
                            safe_ctxt.userData,
                            out.as_mut_ptr(),
                            1,
                        )
                    };
                }
            } else {
                if hex == 'x' as u8 || hex == 'X' as u8 {
                    unsafe {
                        snprintf(
                            out.as_mut_ptr() as *mut i8,
                            size_of::<[xmlChar; 16]>() as u64,
                            b"#x%X\x00" as *const u8 as *const i8,
                            value,
                        )
                    };
                } else {
                    unsafe {
                        snprintf(
                            out.as_mut_ptr() as *mut i8,
                            size_of::<[xmlChar; 16]>() as u64,
                            b"#%d\x00" as *const u8 as *const i8,
                            value,
                        )
                    };
                }
                if !safe_ctxt.sax.is_null()
                    && unsafe { (*safe_ctxt.sax).reference.is_some() }
                    && safe_ctxt.disableSAX == 0
                {
                    unsafe {
                        (*safe_ctxt.sax)
                            .reference
                            .expect("non-null function pointer")(
                            safe_ctxt.userData,
                            out.as_mut_ptr(),
                        )
                    };
                }
            }
        } else {
            /*
             * Just encode the value in UTF-8
             */

            i += unsafe { xmlCopyCharMultiByte(&mut *out.as_mut_ptr().offset(i as isize), value) };

            out[i as usize] = 0;

            if !safe_ctxt.sax.is_null()
                && unsafe { (*safe_ctxt.sax).characters.is_some() }
                && safe_ctxt.disableSAX == 0
            {
                unsafe {
                    (*safe_ctxt.sax)
                        .characters
                        .expect("non-null function pointer")(
                        safe_ctxt.userData,
                        out.as_mut_ptr(),
                        i,
                    )
                };
            }
        }
        return;
    }
    /*
     * We are seeing an entity reference
     */
    ent = xmlParseEntityRef(ctxt);
    if ent.is_null() {
        return;
    }
    let mut safe_ent = unsafe { &mut *ent };
    if safe_ctxt.wellFormed == 0 {
        return;
    }
    was_checked = safe_ent.checked;
    /* special case of predefined entities */
    if safe_ent.name.is_null() || safe_ent.etype == XML_INTERNAL_PREDEFINED_ENTITY {
        val = safe_ent.content;
        if val.is_null() {
            return;
        }
        /*
         * inline the entity.
         */

        if !safe_ctxt.sax.is_null()
            && unsafe { (*safe_ctxt.sax).characters.is_some() }
            && safe_ctxt.disableSAX == 0
        {
            unsafe {
                (*safe_ctxt.sax)
                    .characters
                    .expect("non-null function pointer")(
                    safe_ctxt.userData,
                    val,
                    xmlStrlen_safe(val),
                )
            };
        }

        return;
    }
    /*
     * The first reference to the entity trigger a parsing phase
     * where the ent->children is filled with the result from
     * the parsing.
     * Note: external parsed entities will not be loaded, it is not
     * required for a non-validating parser, unless the parsing option
     * of validating, or substituting entities were given. Doing so is
     * far more secure as the parser will only process data coming from
     * the document entity by default.
     */
    if (safe_ent.checked == 0
        || safe_ent.children.is_null() && safe_ctxt.options & XML_PARSE_NOENT as i32 != 0)
        && (safe_ent.etype != XML_EXTERNAL_GENERAL_PARSED_ENTITY
            || safe_ctxt.options & (XML_PARSE_NOENT as i32 | XML_PARSE_DTDVALID as i32) != 0)
    {
        let oldnbent: u64 = safe_ctxt.nbentities;
        let mut diff: u64 = 0;
        /*
         * This is a bit hackish but this seems the best
         * way to make sure both SAX and DOM entity support
         * behaves okay.
         */
        let user_data: *mut ();
        if safe_ctxt.userData == ctxt as *mut () {
            user_data = 0 as *mut ()
        } else {
            user_data = safe_ctxt.userData
        }
        /*
         * Check that this entity is well formed
         * 4.3.2: An internal general parsed entity is well-formed
         * if its replacement text matches the production labeled
         * content.
         */
        if safe_ent.etype == XML_INTERNAL_GENERAL_ENTITY {
            safe_ctxt.depth += 1;
            ret = xmlParseBalancedChunkMemoryInternal(ctxt, safe_ent.content, user_data, &mut list);
            safe_ctxt.depth -= 1
        } else if safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY {
            safe_ctxt.depth += 1;
            ret = xmlParseExternalEntityPrivate(
                safe_ctxt.myDoc,
                ctxt,
                safe_ctxt.sax,
                user_data,
                safe_ctxt.depth,
                safe_ent.URI,
                safe_ent.ExternalID,
                &mut list,
            );
            safe_ctxt.depth -= 1
        } else {
            ret = XML_ERR_ENTITY_PE_INTERNAL;

            xmlErrMsgStr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"invalid entity type found\n\x00" as *const u8 as *const i8,
                0 as *const xmlChar,
            );
        }
        /*
         * Store the number of entities needing parsing for this entity
         * content and do checkings
         */
        diff = safe_ctxt.nbentities - oldnbent + 1;
        if diff > (INT_MAX / 2) as u64 {
            diff = (INT_MAX / 2) as u64
        }
        safe_ent.checked = (diff * 2) as i32;
        if !safe_ent.content.is_null()
            && !unsafe { xmlStrchr_safe(safe_ent.content, '<' as xmlChar).is_null() }
        {
            safe_ent.checked |= 1
        }
        if ret == XML_ERR_ENTITY_LOOP {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
                xmlHaltParser(ctxt);
            }
            unsafe { xmlFreeNodeList_safe(list) };
            return;
        }
        if xmlParserEntityCheck(ctxt, 0 as size_t, ent, 0 as size_t) != 0 {
            unsafe { xmlFreeNodeList_safe(list) };
            return;
        }
        if ret == XML_ERR_OK && !list.is_null() {
            if (safe_ent.etype == XML_INTERNAL_GENERAL_ENTITY
                || safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY)
                && safe_ent.children.is_null()
            {
                safe_ent.children = list;
                let mut safe_list = unsafe { &mut *list };
                /*
                 * Prune it directly in the generated document
                 * except for single text nodes.
                 */
                if safe_ctxt.replaceEntities == 0
                    || safe_ctxt.parseMode == XML_PARSE_READER
                    || safe_list.type_0 == XML_TEXT_NODE && safe_list.next.is_null()
                {
                    safe_ent.owner = 1;
                    while !list.is_null() {
                        safe_list.parent = ent as xmlNodePtr;
                        unsafe { xmlSetTreeDoc_safe(list, safe_ent.doc) };
                        if safe_list.next.is_null() {
                            safe_ent.last = list
                        }
                        list = safe_list.next;
                        safe_list = unsafe { &mut *list };
                    }
                    list = 0 as xmlNodePtr
                } else {
                    safe_ent.owner = 0;
                    while !list.is_null() {
                        safe_list.parent = safe_ctxt.node;
                        safe_list.doc = safe_ctxt.myDoc;
                        if safe_list.next.is_null() {
                            safe_ent.last = list
                        }
                        list = safe_list.next;
                        safe_list = unsafe { &mut *list };
                    }
                    list = safe_ent.children;

                    match () {
                        #[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
                        _ => {
                            if safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY {
                                xmlAddEntityReference(ent, list, 0 as xmlNodePtr);
                            }
                        }
                        #[cfg(not(HAVE_parser_LIBXML_LEGACY_ENABLED))]
                        _ => {}
                    };

                    /* LIBXML_LEGACY_ENABLED */
                }
            } else {
                unsafe { xmlFreeNodeList_safe(list) };
                list = 0 as xmlNodePtr
            }
        } else if ret != XML_ERR_OK && ret != XML_WAR_UNDECLARED_ENTITY {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_UNDECLARED_ENTITY,
                b"Entity \'%s\' failed to parse\n\x00" as *const u8 as *const i8,
                safe_ent.name,
            );

            if !safe_ent.content.is_null() {
                unsafe { *safe_ent.content.offset(0 as isize) = 0 }
            }
            xmlParserEntityCheck(ctxt, 0 as size_t, ent, 0 as size_t);
        } else if !list.is_null() {
            unsafe { xmlFreeNodeList_safe(list) };
            list = 0 as xmlNodePtr
        }
        if safe_ent.checked == 0 {
            safe_ent.checked = 2
        }
        /* Prevent entity from being parsed and expanded twice (Bug 760367). */
        was_checked = 0
    } else if safe_ent.checked != 1 {
        safe_ctxt.nbentities = safe_ctxt.nbentities + (safe_ent.checked / 2) as u64
    }
    /*
     * Now that the entity content has been gathered
     * provide it to the application, this can take different forms based
     * on the parsing modes.
     */
    if safe_ent.children.is_null() {
        /*
         * Probably running in SAX mode and the callbacks don't
         * build the entity content. So unless we already went
         * though parsing for first checking go though the entity
         * content to generate callbacks associated to the entity
         */
        if was_checked != 0 {
            let user_data: *mut ();
            /*
             * This is a bit hackish but this seems the best
             * way to make sure both SAX and DOM entity support
             * behaves okay.
             */
            if safe_ctxt.userData == ctxt as *mut () {
                user_data = 0 as *mut ()
            } else {
                user_data = safe_ctxt.userData
            }
            if safe_ent.etype == XML_INTERNAL_GENERAL_ENTITY {
                safe_ctxt.depth += 1;
                ret = xmlParseBalancedChunkMemoryInternal(
                    ctxt,
                    safe_ent.content,
                    user_data,
                    0 as *mut xmlNodePtr,
                );
                safe_ctxt.depth -= 1
            } else if safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY {
                safe_ctxt.depth += 1;
                ret = xmlParseExternalEntityPrivate(
                    safe_ctxt.myDoc,
                    ctxt,
                    safe_ctxt.sax,
                    user_data,
                    safe_ctxt.depth,
                    safe_ent.URI,
                    safe_ent.ExternalID,
                    0 as *mut xmlNodePtr,
                );
                safe_ctxt.depth -= 1
            } else {
                ret = XML_ERR_ENTITY_PE_INTERNAL;

                xmlErrMsgStr(
                    ctxt,
                    XML_ERR_INTERNAL_ERROR,
                    b"invalid entity type found\n\x00" as *const u8 as *const i8,
                    0 as *const xmlChar,
                );
            }
            if ret == XML_ERR_ENTITY_LOOP {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
                }
                return;
            }
        }
        if !safe_ctxt.sax.is_null()
            && unsafe { (*safe_ctxt.sax).reference.is_some() }
            && safe_ctxt.replaceEntities == 0
            && safe_ctxt.disableSAX == 0
        {
            /*
             * Entity reference callback comes second, it's somewhat
             * superfluous but a compatibility to historical behaviour
             */
            unsafe {
                (*safe_ctxt.sax)
                    .reference
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, safe_ent.name
                );
            }
        }
        return;
    }

    /*
     * If we didn't get any children for the entity being built
     */
    if !safe_ctxt.sax.is_null()
        && unsafe { (*safe_ctxt.sax).reference.is_some() }
        && safe_ctxt.replaceEntities == 0
        && safe_ctxt.disableSAX == 0
    {
        /*
         * Create a node.
         */
        unsafe {
            (*safe_ctxt.sax)
                .reference
                .expect("non-null function pointer")(safe_ctxt.userData, safe_ent.name);
        }
        return;
    }
    if safe_ctxt.replaceEntities != 0 || safe_ent.children.is_null() {
        /*
         * There is a problem on the handling of _private for entities
         * (bug 155816): Should we copy the content of the field from
         * the entity (possibly overwriting some value set by the user
         * when a copy is created), should we leave it alone, or should
         * we try to take care of different situations?  The problem
         * is exacerbated by the usage of this field by the xmlReader.
         * To fix this bug, we look at _private on the created node
         * and, if it's NULL, we copy in whatever was in the entity.
         * If it's not NULL we leave it alone.  This is somewhat of a
         * hack - maybe we should have further tests to determine
         * what to do.
         */
        if !safe_ctxt.node.is_null() && !safe_ent.children.is_null() {
            /*
             * Seems we are generating the DOM content, do
             * a simple tree copy for all references except the first
             * In the first occurrence list contains the replacement.
             */
            if list.is_null() && safe_ent.owner == 0 || safe_ctxt.parseMode == XML_PARSE_READER {
                let mut nw: xmlNodePtr = 0 as xmlNodePtr;
                let mut cur: xmlNodePtr = 0 as *mut xmlNode;
                let mut firstChild: xmlNodePtr = 0 as xmlNodePtr;
                /* LIBXML_LEGACY_ENABLED */
                safe_ctxt.sizeentcopy = safe_ctxt.sizeentcopy + (safe_ent.length + 5) as u64;
                if xmlParserEntityCheck(ctxt, 0 as size_t, ent, safe_ctxt.sizeentcopy) != 0 {
                    return;
                }
                cur = safe_ent.children;
                while !cur.is_null() {
                    nw = unsafe { xmlDocCopyNode_safe(cur, safe_ctxt.myDoc, 1) };

                    if !nw.is_null() {
                        if unsafe { (*nw)._private.is_null() } {
                            unsafe { (*nw)._private = (*cur)._private }
                        }
                        if firstChild.is_null() {
                            firstChild = nw
                        }
                        nw = unsafe { xmlAddChild_safe(safe_ctxt.node, nw) }
                    }

                    if cur == safe_ent.last {
                        if safe_ctxt.parseMode == XML_PARSE_READER
                            && !nw.is_null()
                            && unsafe {
                                (*nw).type_0 == XML_ELEMENT_NODE && (*nw).children.is_null()
                            }
                        {
                            unsafe { (*nw).extra = 1 }
                        }

                        break;
                    } else {
                        cur = unsafe { (*cur).next };
                    }
                }

                match () {
                    #[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
                    _ => {
                        if safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY {
                            xmlAddEntityReference(ent, firstChild, nw);
                        }
                    }
                    #[cfg(not(HAVE_parser_LIBXML_LEGACY_ENABLEDb))]
                    _ => {}
                };
            } else if list.is_null() || safe_ctxt.inputNr > 0 {
                let mut nw: xmlNodePtr = 0 as xmlNodePtr;
                let mut cur: xmlNodePtr = 0 as *mut xmlNode;
                let mut next: xmlNodePtr = 0 as *mut xmlNode;
                let last: xmlNodePtr;
                let mut firstChild: xmlNodePtr = 0 as xmlNodePtr;
                /* LIBXML_LEGACY_ENABLED */
                safe_ctxt.sizeentcopy = safe_ctxt.sizeentcopy + (safe_ent.length + 5) as u64;
                if xmlParserEntityCheck(ctxt, 0 as size_t, ent, safe_ctxt.sizeentcopy) != 0 {
                    return;
                }
                cur = safe_ent.children;
                safe_ent.children = 0 as *mut _xmlNode;
                last = safe_ent.last;
                safe_ent.last = 0 as *mut _xmlNode;
                while !cur.is_null() {
                    unsafe {
                        next = (*cur).next;
                        (*cur).next = 0 as *mut _xmlNode;
                        (*cur).parent = 0 as *mut _xmlNode;
                        nw = xmlDocCopyNode_safe(cur, safe_ctxt.myDoc, 1 as i32);
                        if !nw.is_null() {
                            if (*nw)._private.is_null() {
                                (*nw)._private = (*cur)._private
                            }
                            if firstChild.is_null() {
                                firstChild = cur
                            }
                            xmlAddChild_safe(ent as xmlNodePtr, nw);
                            xmlAddChild_safe(safe_ctxt.node, cur);
                        }
                    }

                    if cur == last {
                        break;
                    }
                    cur = next
                }
                if safe_ent.owner == 0 {
                    safe_ent.owner = 1
                }
                match () {
                    #[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
                    _ => {
                        if safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY {
                            xmlAddEntityReference(ent, firstChild, nw);
                        }
                    }
                    #[cfg(not(HAVE_parser_LIBXML_LEGACY_ENABLED))]
                    _ => {}
                };
            } else {
                let mut nbktext: *const xmlChar = 0 as *const xmlChar;
                /*
                 * We are copying here, make sure there is no abuse
                 */
                /*
                 * Copy the entity child list and make it the new
                 * entity child list. The goal is to make sure any
                 * ID or REF referenced will be the one from the
                 * document content and not the entity copy.
                 */
                /*
                 * the name change is to avoid coalescing of the
                 * node with a possible previous text one which
                 * would make ent->children a dangling pointer
                 */
                nbktext = unsafe {
                    xmlDictLookup_safe(
                        safe_ctxt.dict,
                        b"nbktext\x00" as *const u8 as *const i8 as *mut xmlChar,
                        -1,
                    )
                };
                unsafe {
                    if (*safe_ent.children).type_0 == XML_TEXT_NODE {
                        (*safe_ent.children).name = nbktext
                    }
                    if safe_ent.last != safe_ent.children
                        && (*safe_ent.last).type_0 == XML_TEXT_NODE
                    {
                        (*safe_ent.last).name = nbktext
                    }
                }
                unsafe { xmlAddChildList_safe(safe_ctxt.node, safe_ent.children) };
            }
            /*
             * This is to avoid a nasty side effect, see
             * characters() in SAX.c
             */
            safe_ctxt.nodemem = 0;
            safe_ctxt.nodelen = 0;
            return;
        }
    };
}
/* *
* xmlParseEntityRef:
* @ctxt:  an XML parser context
*
* parse ENTITY references declarations
*
* [68] EntityRef ::= '&' Name ';'
*
* [ WFC: Entity Declared ]
* In a document without any DTD, a document with only an internal DTD
* subset which contains no parameter entity references, or a document
* with "standalone='yes'", the Name given in the entity reference
* must match that in an entity declaration, except that well-formed
* documents need not declare any of the following entities: amp, lt,
* gt, apos, quot.  The declaration of a parameter entity must precede
* any reference to it.  Similarly, the declaration of a general entity
* must precede any reference to it which appears in a default value in an
* attribute-list declaration. Note that if entities are declared in the
* external subset or in external parameter entities, a non-validating
* processor is not obligated to read and process their declarations;
* for such documents, the rule that an entity must be declared is a
* well-formedness constraint only if standalone='yes'.
*
* [ WFC: Parsed Entity ]
* An entity reference must not contain the name of an unparsed entity
*
* Returns the xmlEntityPtr if found, or NULL otherwise.
*/

pub fn xmlParseEntityRef(mut ctxt: xmlParserCtxtPtr) -> xmlEntityPtr {
    let name: *const xmlChar;
    let mut ent: xmlEntityPtr = 0 as xmlEntityPtr;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let safe_input = unsafe { &mut *safe_ctxt.input };
    if safe_ctxt.progressive == 0
        && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return 0 as xmlEntityPtr;
    }
    if unsafe { *safe_input.cur != '&' as u8 } {
        return 0 as xmlEntityPtr;
    }
    unsafe { xmlNextChar_safe(ctxt) };
    name = xmlParseName(ctxt);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"xmlParseEntityRef: no name\n\x00" as *const u8 as *const i8,
        );

        return 0 as xmlEntityPtr;
    }
    if unsafe { *safe_input.cur != ';' as u8 } {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ENTITYREF_SEMICOL_MISSING, 0 as *const i8);
        }
        return 0 as xmlEntityPtr;
    }
    unsafe { xmlNextChar_safe(ctxt) };
    /*
     * Predefined entities override any extra definition
     */
    if safe_ctxt.options & XML_PARSE_OLDSAX as i32 == 0 {
        ent = unsafe { xmlGetPredefinedEntity_safe(name) };
        if !ent.is_null() {
            return ent;
        }
    }
    /*
     * Increase the number of entity references parsed
     */
    safe_ctxt.nbentities = safe_ctxt.nbentities + 1;
    /*
     * Ask first SAX for entity resolution, otherwise try the
     * entities which may have stored in the parser context.
     */
    if !safe_ctxt.sax.is_null() {
        unsafe {
            if (*safe_ctxt.sax).getEntity.is_some() {
                ent = (*safe_ctxt.sax)
                    .getEntity
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, name
                )
            }
        }
        if safe_ctxt.wellFormed == 1
            && ent.is_null()
            && safe_ctxt.options & XML_PARSE_OLDSAX as i32 != 0
        {
            ent = unsafe { xmlGetPredefinedEntity_safe(name) }
        }
        if safe_ctxt.wellFormed == 1 && ent.is_null() && safe_ctxt.userData == ctxt as *mut () {
            ent = unsafe { xmlSAX2GetEntity_safe(ctxt as *mut (), name) }
        }
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return 0 as xmlEntityPtr;
    }
    /*
     * [ WFC: Entity Declared ]
     * In a document without any DTD, a document with only an
     * internal DTD subset which contains no parameter entity
     * references, or a document with "standalone='yes'", the
     * Name given in the entity reference must match that in an
     * entity declaration, except that well-formed documents
     * need not declare any of the following entities: amp, lt,
     * gt, apos, quot.
     * The declaration of a parameter entity must precede any
     * reference to it.
     * Similarly, the declaration of a general entity must
     * precede any reference to it which appears in a default
     * value in an attribute-list declaration. Note that if
     * entities are declared in the external subset or in
     * external parameter entities, a non-validating processor
     * is not obligated to read and process their declarations;
     * for such documents, the rule that an entity must be
     * declared is a well-formedness constraint only if
     * standalone='yes'.
     */
    let mut safe_ent = unsafe { &mut *ent };
    if ent.is_null() {
        if safe_ctxt.standalone == 1 || safe_ctxt.hasExternalSubset == 0 && safe_ctxt.hasPErefs == 0
        {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_UNDECLARED_ENTITY,
                b"Entity \'%s\' not defined\n\x00" as *const u8 as *const i8,
                name,
            );
        } else {
            unsafe {
                xmlErrMsgStr(
                    ctxt,
                    XML_WAR_UNDECLARED_ENTITY,
                    b"Entity \'%s\' not defined\n\x00" as *const u8 as *const i8,
                    name,
                );
                if safe_ctxt.inSubset == 0
                    && !safe_ctxt.sax.is_null()
                    && (*safe_ctxt.sax).reference.is_some()
                {
                    (*safe_ctxt.sax)
                        .reference
                        .expect("non-null function pointer")(
                        safe_ctxt.userData, name
                    );
                }
            }
        }
        xmlParserEntityCheck(ctxt, 0 as size_t, ent, 0 as size_t);
        safe_ctxt.valid = 0
    } else if safe_ent.etype == XML_EXTERNAL_GENERAL_UNPARSED_ENTITY {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_UNPARSED_ENTITY,
            b"Entity reference to unparsed entity %s\n\x00" as *const u8 as *const i8,
            name,
        );
    } else if safe_ctxt.instate == XML_PARSER_ATTRIBUTE_VALUE as i32
        && safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY
    {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_ENTITY_IS_EXTERNAL,
            b"Attribute references external entity \'%s\'\n\x00" as *const u8 as *const i8,
            name,
        );
    } else if safe_ctxt.instate == XML_PARSER_ATTRIBUTE_VALUE as i32
        && !ent.is_null()
        && safe_ent.etype != XML_INTERNAL_PREDEFINED_ENTITY
    {
        if (safe_ent.checked & 1 != 0 || safe_ent.checked == 0)
            && !safe_ent.content.is_null()
            && !unsafe { xmlStrchr_safe(safe_ent.content, '<' as u8 as xmlChar).is_null() }
        {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_LT_IN_ATTRIBUTE,
                b"\'<\' in entity \'%s\' is not allowed in attributes values\n\x00" as *const u8
                    as *const i8,
                name,
            );
        }
    } else {
        /*
         * [ WFC: Parsed Entity ]
         * An entity reference must not contain the name of an
         * unparsed entity
         */
        /*
         * [ WFC: No External Entity References ]
         * Attribute values cannot contain direct or indirect
         * entity references to external entities.
         */
        /*
         * [ WFC: No < in Attribute Values ]
         * The replacement text of any entity referred to directly or
         * indirectly in an attribute value (other than "&lt;") must
         * not contain a <.
         */
        /*
         * Internal check, no parameter entities here ...
         */
        match safe_ent.etype {
            XML_INTERNAL_PARAMETER_ENTITY | XML_EXTERNAL_PARAMETER_ENTITY => {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_ENTITY_IS_PARAMETER,
                    b"Attempt to reference the parameter entity \'%s\'\n\x00" as *const u8
                        as *const i8,
                    name,
                );
            }
            _ => {}
        }
    }
    /*
     * [ WFC: No Recursion ]
     * A parsed entity must not contain a recursive reference
     * to itself, either directly or indirectly.
     * Done somewhere else
     */
    return ent;
}
/* ***********************************************************************
*									*
*		Parser stacks related functions and macros		*
*									*
************************************************************************/
/* *
* xmlParseStringEntityRef:
* @ctxt:  an XML parser context
* @str:  a pointer to an index in the string
*
* parse ENTITY references declarations, but this version parses it from
* a string value.
*
* [68] EntityRef ::= '&' Name ';'
*
* [ WFC: Entity Declared ]
* In a document without any DTD, a document with only an internal DTD
* subset which contains no parameter entity references, or a document
* with "standalone='yes'", the Name given in the entity reference
* must match that in an entity declaration, except that well-formed
* documents need not declare any of the following entities: amp, lt,
* gt, apos, quot.  The declaration of a parameter entity must precede
* any reference to it.  Similarly, the declaration of a general entity
* must precede any reference to it which appears in a default value in an
* attribute-list declaration. Note that if entities are declared in the
* external subset or in external parameter entities, a non-validating
* processor is not obligated to read and process their declarations;
* for such documents, the rule that an entity must be declared is a
* well-formedness constraint only if standalone='yes'.
*
* [ WFC: Parsed Entity ]
* An entity reference must not contain the name of an unparsed entity
*
* Returns the xmlEntityPtr if found, or NULL otherwise. The str pointer
* is updated to the current location in the string.
*/
fn xmlParseStringEntityRef(
    mut ctxt: xmlParserCtxtPtr,
    mut str: *mut *const xmlChar,
) -> xmlEntityPtr {
    let name: *mut xmlChar;
    let mut ptr: *const xmlChar = 0 as *const xmlChar;
    let mut cur: xmlChar = 0;
    let mut ent: xmlEntityPtr = 0 as xmlEntityPtr;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if str.is_null() || unsafe { (*str).is_null() } {
        return 0 as xmlEntityPtr;
    }
    unsafe {
        ptr = *str;
        cur = *ptr;
    }
    if cur != '&' as u8 {
        return 0 as xmlEntityPtr;
    }
    ptr = unsafe { ptr.offset(1) };
    name = xmlParseStringName(ctxt, &mut ptr);
    if name.is_null() {
        unsafe {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_NAME_REQUIRED,
                b"xmlParseStringEntityRef: no name\n\x00" as *const u8 as *const i8,
            );
            *str = ptr;
        }
        return 0 as xmlEntityPtr;
    }
    unsafe {
        if *ptr != ';' as u8 {
            xmlFatalErr(ctxt, XML_ERR_ENTITYREF_SEMICOL_MISSING, 0 as *const i8);
            xmlFree_safe(name as *mut ());
            *str = ptr;
            return 0 as xmlEntityPtr;
        }
        ptr = ptr.offset(1);
    }
    /*
     * Predefined entities override any extra definition
     */
    if safe_ctxt.options & XML_PARSE_OLDSAX as i32 == 0 {
        ent = unsafe { xmlGetPredefinedEntity_safe(name) };
        if !ent.is_null() {
            unsafe { xmlFree_safe(name as *mut ()) };
            unsafe {
                *str = ptr;
            }
            return ent;
        }
    }
    /*
     * Increase the number of entity references parsed
     */
    safe_ctxt.nbentities = safe_ctxt.nbentities + 1;
    /*
     * Ask first SAX for entity resolution, otherwise try the
     * entities which may have stored in the parser context.
     */
    if !safe_ctxt.sax.is_null() {
        unsafe {
            if (*safe_ctxt.sax).getEntity.is_some() {
                ent = (*safe_ctxt.sax)
                    .getEntity
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, name
                )
            }
        }
        if ent.is_null() && safe_ctxt.options & XML_PARSE_OLDSAX as i32 != 0 {
            ent = unsafe { xmlGetPredefinedEntity_safe(name) }
        }
        if ent.is_null() && safe_ctxt.userData == ctxt as *mut () {
            ent = unsafe { xmlSAX2GetEntity_safe(ctxt as *mut (), name) }
        }
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        unsafe { xmlFree_safe(name as *mut ()) };
        return 0 as xmlEntityPtr;
    }
    /*
     * [ WFC: Entity Declared ]
     * In a document without any DTD, a document with only an
     * internal DTD subset which contains no parameter entity
     * references, or a document with "standalone='yes'", the
     * Name given in the entity reference must match that in an
     * entity declaration, except that well-formed documents
     * need not declare any of the following entities: amp, lt,
     * gt, apos, quot.
     * The declaration of a parameter entity must precede any
     * reference to it.
     * Similarly, the declaration of a general entity must
     * precede any reference to it which appears in a default
     * value in an attribute-list declaration. Note that if
     * entities are declared in the external subset or in
     * external parameter entities, a non-validating processor
     * is not obligated to read and process their declarations;
     * for such documents, the rule that an entity must be
     * declared is a well-formedness constraint only if
     * standalone='yes'.
     */
    let mut safe_ent = unsafe { &mut *ent };
    if ent.is_null() {
        if safe_ctxt.standalone == 1 || safe_ctxt.hasExternalSubset == 0 && safe_ctxt.hasPErefs == 0
        {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_UNDECLARED_ENTITY,
                b"Entity \'%s\' not defined\n\x00" as *const u8 as *const i8,
                name,
            );
        } else {
            xmlErrMsgStr(
                ctxt,
                XML_WAR_UNDECLARED_ENTITY,
                b"Entity \'%s\' not defined\n\x00" as *const u8 as *const i8,
                name,
            );
        }
        xmlParserEntityCheck(ctxt, 0 as size_t, ent, 0 as size_t);
    /* TODO ? check regressions ctxt->valid = 0; */
    } else if safe_ent.etype == XML_EXTERNAL_GENERAL_UNPARSED_ENTITY {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_UNPARSED_ENTITY,
            b"Entity reference to unparsed entity %s\n\x00" as *const u8 as *const i8,
            name,
        );
    } else if safe_ctxt.instate == XML_PARSER_ATTRIBUTE_VALUE as i32
        && safe_ent.etype == XML_EXTERNAL_GENERAL_PARSED_ENTITY
    {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_ENTITY_IS_EXTERNAL,
            b"Attribute references external entity \'%s\'\n\x00" as *const u8 as *const i8,
            name,
        );
    } else if safe_ctxt.instate == XML_PARSER_ATTRIBUTE_VALUE as i32
        && !ent.is_null()
        && !safe_ent.content.is_null()
        && safe_ent.etype != XML_INTERNAL_PREDEFINED_ENTITY
        && !unsafe { xmlStrchr_safe(safe_ent.content, '<' as xmlChar).is_null() }
    {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_LT_IN_ATTRIBUTE,
            b"\'<\' in entity \'%s\' is not allowed in attributes values\n\x00" as *const u8
                as *const i8,
            name,
        );
    } else {
        /*
         * [ WFC: Parsed Entity ]
         * An entity reference must not contain the name of an
         * unparsed entity
         */
        /*
         * [ WFC: No External Entity References ]
         * Attribute values cannot contain direct or indirect
         * entity references to external entities.
         */
        /*
         * [ WFC: No < in Attribute Values ]
         * The replacement text of any entity referred to directly or
         * indirectly in an attribute value (other than "&lt;") must
         * not contain a <.
         */
        /*
         * Internal check, no parameter entities here ...
         */
        match safe_ent.etype {
            XML_INTERNAL_PARAMETER_ENTITY | XML_EXTERNAL_PARAMETER_ENTITY => {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_ENTITY_IS_PARAMETER,
                    b"Attempt to reference the parameter entity \'%s\'\n\x00" as *const u8
                        as *const i8,
                    name,
                );
            }
            _ => {}
        }
    }
    /*
     * [ WFC: No Recursion ]
     * A parsed entity must not contain a recursive reference
     * to itself, either directly or indirectly.
     * Done somewhere else
     */
    unsafe { xmlFree_safe(name as *mut ()) };
    unsafe {
        *str = ptr;
    }
    return ent;
}
/* *
* xmlParsePEReference:
* @ctxt:  an XML parser context
*
* parse PEReference declarations
* The entity content is handled directly by pushing it's content as
* a new input stream.
*
* [69] PEReference ::= '%' Name ';'
*
* [ WFC: No Recursion ]
* A parsed entity must not contain a recursive
* reference to itself, either directly or indirectly.
*
* [ WFC: Entity Declared ]
* In a document without any DTD, a document with only an internal DTD
* subset which contains no parameter entity references, or a document
* with "standalone='yes'", ...  ... The declaration of a parameter
* entity must precede any reference to it...
*
* [ VC: Entity Declared ]
* In a document with an external subset or external parameter entities
* with "standalone='no'", ...  ... The declaration of a parameter entity
* must precede any reference to it...
*
* [ WFC: In DTD ]
* Parameter-entity references may only appear in the DTD.
* NOTE: misleading but this is handled.
*/

pub fn xmlParsePEReference(mut ctxt: xmlParserCtxtPtr) {
    let name: *const xmlChar;
    let mut entity: xmlEntityPtr = 0 as xmlEntityPtr;
    let input: xmlParserInputPtr;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*safe_ctxt.input).cur != '%' as u8 } {
        return;
    }
    unsafe { xmlNextChar_safe(ctxt) };
    name = xmlParseName(ctxt);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_PEREF_NO_NAME,
            b"PEReference: no name\n\x00" as *const u8 as *const i8,
        );

        return;
    }
    unsafe {
        if *__xmlParserDebugEntities() != 0 {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"PEReference: %s\n\x00" as *const u8 as *const i8,
                name,
            );
        }
        if *(*safe_ctxt.input).cur != ';' as u8 {
            xmlFatalErr(ctxt, XML_ERR_PEREF_SEMICOL_MISSING, 0 as *const i8);
            return;
        }
    }

    unsafe { xmlNextChar_safe(ctxt) };
    /*
     * Increase the number of entity references parsed
     */
    safe_ctxt.nbentities = safe_ctxt.nbentities + 1;
    /*
     * Request the entity from SAX
     */
    unsafe {
        if !safe_ctxt.sax.is_null() && (*safe_ctxt.sax).getParameterEntity.is_some() {
            entity = (*safe_ctxt.sax)
                .getParameterEntity
                .expect("non-null function pointer")(safe_ctxt.userData, name)
        }
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return;
    }
    let mut safe_entity = unsafe { &mut *entity };
    if entity.is_null() {
        /*
         * [ WFC: Entity Declared ]
         * In a document without any DTD, a document with only an
         * internal DTD subset which contains no parameter entity
         * references, or a document with "standalone='yes'", ...
         * ... The declaration of a parameter entity must precede
         * any reference to it...
         */
        if safe_ctxt.standalone == 1 || safe_ctxt.hasExternalSubset == 0 && safe_ctxt.hasPErefs == 0
        {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_UNDECLARED_ENTITY,
                b"PEReference: %%%s; not found\n\x00" as *const u8 as *const i8,
                name,
            );
        } else {
            /*
             * [ VC: Entity Declared ]
             * In a document with an external subset or external
             * parameter entities with "standalone='no'", ...
             * ... The declaration of a parameter entity must
             * precede any reference to it...
             */
            if safe_ctxt.validate != 0 && safe_ctxt.vctxt.error.is_some() {
                xmlValidityError(
                    ctxt,
                    XML_WAR_UNDECLARED_ENTITY,
                    b"PEReference: %%%s; not found\n\x00" as *const u8 as *const i8,
                    name,
                    0 as *const xmlChar,
                );
            } else {
                xmlWarningMsg(
                    ctxt,
                    XML_WAR_UNDECLARED_ENTITY,
                    b"PEReference: %%%s; not found\n\x00" as *const u8 as *const i8,
                    name,
                    0 as *const xmlChar,
                );
            }
            safe_ctxt.valid = 0
        }
        xmlParserEntityCheck(ctxt, 0 as size_t, 0 as xmlEntityPtr, 0 as size_t);
    } else if safe_entity.etype != XML_INTERNAL_PARAMETER_ENTITY
        && safe_entity.etype != XML_EXTERNAL_PARAMETER_ENTITY
    {
        xmlWarningMsg(
            ctxt,
            XML_WAR_UNDECLARED_ENTITY,
            b"Internal: %%%s; is not a parameter entity\n\x00" as *const u8 as *const i8,
            name,
            0 as *const xmlChar,
        );
    } else {
        let mut start: [xmlChar; 4] = [0; 4];
        let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
        if xmlParserEntityCheck(ctxt, 0 as size_t, entity, 0 as size_t) != 0 {
            return;
        }
        if safe_entity.etype == XML_EXTERNAL_PARAMETER_ENTITY
            && safe_ctxt.options & XML_PARSE_NOENT as i32 == 0
            && safe_ctxt.options & XML_PARSE_DTDVALID as i32 == 0
            && safe_ctxt.options & XML_PARSE_DTDLOAD as i32 == 0
            && safe_ctxt.options & XML_PARSE_DTDATTR as i32 == 0
            && safe_ctxt.replaceEntities == 0
            && safe_ctxt.validate == 0
        {
            return;
        }
        input = xmlNewEntityInputStream(ctxt, entity);
        if xmlPushInput(ctxt, input) < 0 {
            unsafe { xmlFreeInputStream_safe(input) };
            return;
        }
        if safe_entity.etype == XML_EXTERNAL_PARAMETER_ENTITY {
            /*
             * Internal checking in case the entity quest barfed
             */
            /*
             * Get the 4 first bytes and decode the charset
             * if enc != XML_CHAR_ENCODING_NONE
             * plug some encoding conversion routines.
             * Note that, since we may have some non-UTF8
             * encoding (like UTF16, bug 135229), the 'length'
             * is not known, but we can calculate based upon
             * the amount of data in the buffer.
             */
            if safe_ctxt.progressive == 0
                && unsafe {
                    ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
                }
            {
                xmlGROW(ctxt);
            }
            if safe_ctxt.instate == XML_PARSER_EOF {
                return;
            }
            unsafe {
                if (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 >= 4 {
                    start[0] = *(*safe_ctxt.input).cur;
                    start[1] = *(*safe_ctxt.input).cur.offset(1);
                    start[2] = *(*safe_ctxt.input).cur.offset(2);
                    start[3] = *(*safe_ctxt.input).cur.offset(3);
                    enc = xmlDetectCharEncoding_safe(start.as_mut_ptr(), 4);
                    if enc != XML_CHAR_ENCODING_NONE {
                        xmlSwitchEncoding_safe(ctxt, enc);
                    }
                }
                if *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '?' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(2) == 'x' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'm' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'l' as u8
                    && (*(*safe_ctxt.input).cur.offset(5) as i32 == 0x20
                        || 0x9 <= *(*safe_ctxt.input).cur.offset(5) as i32
                            && *(*safe_ctxt.input).cur.offset(5) as i32 <= 0xa
                        || *(*safe_ctxt.input).cur.offset(5) as i32 == 0xd)
                {
                    xmlParseTextDecl(ctxt);
                }
            }
        }
    }
    safe_ctxt.hasPErefs = 1;
}
/* *
* xmlLoadEntityContent:
* @ctxt:  an XML parser context
* @entity: an unloaded system entity
*
* Load the original content of the given system entity from the
* ExternalID/SystemID given. This is to be used for Included in Literal
* http://www.w3.org/TR/REC-xml/#inliteral processing of entities references
*
* Returns 0 in case of success and -1 in case of failure
*/
fn xmlLoadEntityContent(mut ctxt: xmlParserCtxtPtr, mut entity: xmlEntityPtr) -> i32 {
    let input: xmlParserInputPtr;
    let mut buf: xmlBufferPtr = 0 as *mut xmlBuffer;
    let mut l: i32 = 0;
    let mut c: i32 = 0;
    let mut count: i32 = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_entity = unsafe { &mut *entity };
    if ctxt.is_null()
        || entity.is_null()
        || safe_entity.etype != XML_EXTERNAL_PARAMETER_ENTITY
            && safe_entity.etype != XML_EXTERNAL_GENERAL_PARSED_ENTITY
        || !safe_entity.content.is_null()
    {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"xmlLoadEntityContent parameter error\x00" as *const u8 as *const i8,
            );
        }
        return -1;
    }
    unsafe {
        if *__xmlParserDebugEntities() != 0 {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"Reading %s entity content input\n\x00" as *const u8 as *const i8,
                safe_entity.name,
            );
        }
    }
    buf = unsafe { xmlBufferCreate_safe() };
    if buf.is_null() {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"xmlLoadEntityContent parameter error\x00" as *const u8 as *const i8,
            );
        }
        return -1;
    }
    input = xmlNewEntityInputStream(ctxt, entity);
    if input.is_null() {
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"xmlLoadEntityContent input error\x00" as *const u8 as *const i8,
            );
        }
        unsafe { xmlBufferFree_safe(buf) };
        return -1;
    }
    /*
     * Push the entity as the current input, read char by char
     * saving to the buffer until the end of the entity or an error
     */
    if xmlPushInput(ctxt, input) < 0 {
        unsafe { xmlBufferFree_safe(buf) };
        return -1;
    }
    if safe_ctxt.progressive == 0
        && unsafe { ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }

    c = xmlCurrentChar(ctxt, &mut l);

    let safe_input = unsafe { &mut *safe_ctxt.input };
    while safe_ctxt.input == input
        && safe_input.cur < safe_input.end
        && (if c < 0x100 {
            (0x9 <= c && c <= 0xa || c == 0xd || 0x20 <= c) as i32
        } else {
            (0x100 <= c && c <= 0xd7ff
                || 0xe000 <= c && c <= 0xfffd
                || 0x10000 <= c && c <= 0x10ffff) as i32
        }) != 0
    {
        unsafe { xmlBufferAdd_safe(buf, safe_input.cur, l) };
        let count_old = count;
        count = count + 1;
        if count_old > 100 {
            count = 0;
            if safe_ctxt.progressive == 0
                && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
            {
                xmlGROW(ctxt);
            }
            if safe_ctxt.instate == XML_PARSER_EOF {
                unsafe { xmlBufferFree_safe(buf) };
                return -1;
            }
        }

        if unsafe { *safe_input.cur == '\n' as u8 } {
            safe_input.line += 1;
            safe_input.col = 1
        } else {
            safe_input.col += 1
        }
        safe_input.cur = unsafe { safe_input.cur.offset(l as isize) };
        c = xmlCurrentChar(ctxt, &mut l);

        if c == 0 {
            count = 0;

            if safe_ctxt.progressive == 0
                && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
            {
                xmlGROW(ctxt);
            }
            if safe_ctxt.instate == XML_PARSER_EOF {
                unsafe { xmlBufferFree_safe(buf) };
                return -1;
            }

            c = xmlCurrentChar(ctxt, &mut l);
        }
    }
    if safe_ctxt.input == input && safe_input.cur >= safe_input.end {
        xmlPopInput_parser(ctxt);
    } else if if c < 0x100 {
        (0x9 <= c && c <= 0xa || c == 0xd || 0x20 <= c) as i32
    } else {
        (0x100 <= c && c <= 0xd7ff || 0xe000 <= c && c <= 0xfffd || 0x10000 <= c && c <= 0x10ffff)
            as i32
    } == 0
    {
        xmlFatalErrMsgInt(
            ctxt,
            XML_ERR_INVALID_CHAR,
            b"xmlLoadEntityContent: invalid char value %d\n\x00" as *const u8 as *const i8,
            c,
        );

        unsafe { xmlBufferFree_safe(buf) };
        return -1;
    }
    unsafe {
        safe_entity.content = (*buf).content;
        (*buf).content = 0 as *mut xmlChar;
    }
    unsafe { xmlBufferFree_safe(buf) };
    return 0;
}
/* DEPR void xmlParserHandleReference(xmlParserCtxtPtr ctxt); */
/* *
* xmlParseStringPEReference:
* @ctxt:  an XML parser context
* @str:  a pointer to an index in the string
*
* parse PEReference declarations
*
* [69] PEReference ::= '%' Name ';'
*
* [ WFC: No Recursion ]
* A parsed entity must not contain a recursive
* reference to itself, either directly or indirectly.
*
* [ WFC: Entity Declared ]
* In a document without any DTD, a document with only an internal DTD
* subset which contains no parameter entity references, or a document
* with "standalone='yes'", ...  ... The declaration of a parameter
* entity must precede any reference to it...
*
* [ VC: Entity Declared ]
* In a document with an external subset or external parameter entities
* with "standalone='no'", ...  ... The declaration of a parameter entity
* must precede any reference to it...
*
* [ WFC: In DTD ]
* Parameter-entity references may only appear in the DTD.
* NOTE: misleading but this is handled.
*
* Returns the string of the entity content.
*         str is updated to the current value of the index
*/
fn xmlParseStringPEReference(
    mut ctxt: xmlParserCtxtPtr,
    mut str: *mut *const xmlChar,
) -> xmlEntityPtr {
    let mut ptr: *const xmlChar = 0 as *const xmlChar;
    let mut cur: xmlChar = 0;
    let name: *mut xmlChar;
    let mut entity: xmlEntityPtr = 0 as xmlEntityPtr;
    if str.is_null() || unsafe { (*str).is_null() } {
        return 0 as xmlEntityPtr;
    }
    unsafe {
        ptr = *str;
        cur = *ptr;
    }
    if cur != '%' as u8 {
        return 0 as xmlEntityPtr;
    }
    ptr = unsafe { ptr.offset(1) };
    name = xmlParseStringName(ctxt, &mut ptr);
    if name.is_null() {
        unsafe {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_NAME_REQUIRED,
                b"xmlParseStringPEReference: no name\n\x00" as *const u8 as *const i8,
            );
            *str = ptr;
        }
        return 0 as xmlEntityPtr;
    }
    cur = unsafe { *ptr };
    if cur != ';' as u8 {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_ENTITYREF_SEMICOL_MISSING, 0 as *const i8);
        }
        unsafe { xmlFree_safe(name as *mut ()) };
        unsafe {
            *str = ptr;
        }
        return 0 as xmlEntityPtr;
    }
    ptr = unsafe { ptr.offset(1) };
    let mut safe_ctxt = unsafe { &mut *ctxt };
    /*
     * Increase the number of entity references parsed
     */
    safe_ctxt.nbentities = safe_ctxt.nbentities + 1;
    /*
     * Request the entity from SAX
     */
    unsafe {
        if !safe_ctxt.sax.is_null() && (*safe_ctxt.sax).getParameterEntity.is_some() {
            entity = (*safe_ctxt.sax)
                .getParameterEntity
                .expect("non-null function pointer")(safe_ctxt.userData, name)
        }
    }
    let mut safe_entity = unsafe { &mut *entity };
    if safe_ctxt.instate == XML_PARSER_EOF {
        unsafe { xmlFree_safe(name as *mut ()) };
        unsafe { *str = ptr };
        return 0 as xmlEntityPtr;
    }
    if entity.is_null() {
        /*
         * [ WFC: Entity Declared ]
         * In a document without any DTD, a document with only an
         * internal DTD subset which contains no parameter entity
         * references, or a document with "standalone='yes'", ...
         * ... The declaration of a parameter entity must precede
         * any reference to it...
         */
        if safe_ctxt.standalone == 1 || safe_ctxt.hasExternalSubset == 0 && safe_ctxt.hasPErefs == 0
        {
            xmlFatalErrMsgStr(
                ctxt,
                XML_ERR_UNDECLARED_ENTITY,
                b"PEReference: %%%s; not found\n\x00" as *const u8 as *const i8,
                name,
            );
        } else {
            /*
             * [ VC: Entity Declared ]
             * In a document with an external subset or external
             * parameter entities with "standalone='no'", ...
             * ... The declaration of a parameter entity must
             * precede any reference to it...
             */

            xmlWarningMsg(
                ctxt,
                XML_WAR_UNDECLARED_ENTITY,
                b"PEReference: %%%s; not found\n\x00" as *const u8 as *const i8,
                name,
                0 as *const xmlChar,
            );

            safe_ctxt.valid = 0
        }
        xmlParserEntityCheck(ctxt, 0 as size_t, 0 as xmlEntityPtr, 0 as size_t);
    } else if safe_entity.etype != XML_INTERNAL_PARAMETER_ENTITY
        && safe_entity.etype != XML_EXTERNAL_PARAMETER_ENTITY
    {
        xmlWarningMsg(
            ctxt,
            XML_WAR_UNDECLARED_ENTITY,
            b"%%%s; is not a parameter entity\n\x00" as *const u8 as *const i8,
            name,
            0 as *const xmlChar,
        );
    }
    safe_ctxt.hasPErefs = 1;
    unsafe { xmlFree_safe(name as *mut ()) };
    unsafe {
        *str = ptr;
    }
    return entity;
}
/*
* Internal checking in case the entity quest barfed
*/
/* *
* xmlParseDocTypeDecl:
* @ctxt:  an XML parser context
*
* parse a DOCTYPE declaration
*
* [28] doctypedecl ::= '<!DOCTYPE' S Name (S ExternalID)? S?
*                      ('[' (markupdecl | PEReference | S)* ']' S?)? '>'
*
* [ VC: Root Element Type ]
* The Name in the document type declaration must match the element
* type of the root element.
*/

pub fn xmlParseDocTypeDecl(mut ctxt: xmlParserCtxtPtr) {
    let name: *const xmlChar;
    let mut ExternalID: *mut xmlChar = 0 as *mut xmlChar;
    let mut URI: *mut xmlChar = 0 as *mut xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    /*
     * We know that '<!DOCTYPE' has been detected.
     */
    unsafe {
        (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(9);
        (*safe_ctxt.input).col += 9;
        if *(*safe_ctxt.input).cur as i32 == 0 {
            xmlParserInputGrow_safe(safe_ctxt.input, 250);
        }
    }
    xmlSkipBlankChars(ctxt);
    /*
     * Parse the DOCTYPE name.
     */
    name = xmlParseName(ctxt);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"xmlParseDocTypeDecl : no DOCTYPE name !\n\x00" as *const u8 as *const i8,
        );
    }
    safe_ctxt.intSubName = name;
    xmlSkipBlankChars(ctxt);
    /*
     * Check for SystemID and ExternalID
     */
    URI = xmlParseExternalID(ctxt, &mut ExternalID, 1);
    if !URI.is_null() || !ExternalID.is_null() {
        safe_ctxt.hasExternalSubset = 1
    }
    safe_ctxt.extSubURI = URI;
    safe_ctxt.extSubSystem = ExternalID;
    xmlSkipBlankChars(ctxt);
    /*
     * Create and update the internal subset.
     */

    if !safe_ctxt.sax.is_null()
        && unsafe { (*safe_ctxt.sax).internalSubset.is_some() }
        && safe_ctxt.disableSAX == 0
    {
        unsafe {
            (*safe_ctxt.sax)
                .internalSubset
                .expect("non-null function pointer")(
                safe_ctxt.userData, name, ExternalID, URI
            )
        };
    }

    if safe_ctxt.instate == XML_PARSER_EOF {
        return;
    }
    /*
     * Is there any internal subset declarations ?
     * they are handled separately in xmlParseInternalSubset()
     */
    unsafe {
        if *(*safe_ctxt.input).cur == '[' as u8 {
            return;
        }
        /*
         * We should be at the end of the DOCTYPE declaration.
         */
        if *(*safe_ctxt.input).cur != '>' as u8 {
            xmlFatalErr(ctxt, XML_ERR_DOCTYPE_NOT_FINISHED, 0 as *const i8);
        }
    }
    unsafe { xmlNextChar_safe(ctxt) };
}
/* *
* xmlParseInternalSubset:
* @ctxt:  an XML parser context
*
* parse the internal subset declaration
*
* [28 end] ('[' (markupdecl | PEReference | S)* ']' S?)? '>'
*/
fn xmlParseInternalSubset(mut ctxt: xmlParserCtxtPtr) {
    /*
     * Is there any DTD definition ?
     */
    let safe_ctxt = unsafe { &mut *ctxt };
    if unsafe { *(*safe_ctxt.input).cur == '[' as u8 } {
        let mut baseInputNr: i32 = safe_ctxt.inputNr;
        safe_ctxt.instate = XML_PARSER_DTD;
        unsafe { xmlNextChar_safe(ctxt) };
        /*
         * Parse the succession of Markup declarations and
         * PEReferences.
         * Subsequence (markupdecl | PEReference | S)*
         */
        while 1 < 2 {
            if !((unsafe { *(*safe_ctxt.input).cur != ']' as u8 }
                || safe_ctxt.inputNr > baseInputNr)
                && safe_ctxt.instate != XML_PARSER_EOF)
            {
                break;
            }
            let mut check: *const xmlChar = unsafe { (*safe_ctxt.input).cur };
            let mut cons: u32 = unsafe { (*safe_ctxt.input).consumed as u32 };
            xmlSkipBlankChars(ctxt);
            xmlParseMarkupDecl(ctxt);
            xmlParsePEReference(ctxt);
            /*
             * Conditional sections are allowed from external entities included
             * by PE References in the internal subset.
             */
            if safe_ctxt.inputNr > 1
                && unsafe {
                    !(*safe_ctxt.input).filename.is_null()
                        && *(*safe_ctxt.input).cur == '<' as u8
                        && *(*safe_ctxt.input).cur.offset(1) == '!' as u8
                        && *(*safe_ctxt.input).cur.offset(2) == '[' as u8
                }
            {
                xmlParseConditionalSections(ctxt);
            }
            if !(unsafe { (*safe_ctxt.input).cur == check }
                && cons as u64 == unsafe { (*safe_ctxt.input).consumed })
            {
                continue;
            }
            unsafe {
                xmlFatalErr(
                    ctxt,
                    XML_ERR_INTERNAL_ERROR,
                    b"xmlParseInternalSubset: error detected in Markup declaration\n\x00"
                        as *const u8 as *const i8,
                )
            };
            if !(safe_ctxt.inputNr > baseInputNr) {
                break;
            }
            unsafe { xmlPopInput(ctxt) };
        }
        if unsafe { *(*safe_ctxt.input).cur == ']' as u8 } {
            unsafe { xmlNextChar_safe(ctxt) };
            xmlSkipBlankChars(ctxt);
        }
    }
    /*
     * We should be at the end of the DOCTYPE declaration.
     */
    if unsafe { *(*safe_ctxt.input).cur != '>' as u8 } {
        unsafe { xmlFatalErr(ctxt, XML_ERR_DOCTYPE_NOT_FINISHED, 0 as *const i8) };
        return;
    }

    unsafe { xmlNextChar_safe(ctxt) };
}
/* *
* xmlParseAttribute:
* @ctxt:  an XML parser context
* @value:  a xmlChar ** used to store the value of the attribute
*
* parse an attribute
*
* [41] Attribute ::= Name Eq AttValue
*
* [ WFC: No External Entity References ]
* Attribute values cannot contain direct or indirect entity references
* to external entities.
*
* [ WFC: No < in Attribute Values ]
* The replacement text of any entity referred to directly or indirectly in
* an attribute value (other than "&lt;") must not contain a <.
*
* [ VC: Attribute Value Type ]
* The attribute must have been declared; the value must be of the type
* declared for it.
*
* [25] Eq ::= S? '=' S?
*
* With namespace:
*
* [NS 11] Attribute ::= QName Eq AttValue
*
* Also the case QName == xmlns:??? is handled independently as a namespace
* definition.
*
* Returns the attribute name, and the value in *value.
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseAttribute(
    mut ctxt: xmlParserCtxtPtr,
    mut value: *mut *mut xmlChar,
) -> *const xmlChar {
    let name: *const xmlChar;
    let val: *mut xmlChar;
    unsafe { *value = 0 as *mut xmlChar };
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if unsafe {
        safe_ctxt.progressive == 0
            && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    } {
        xmlGROW(ctxt);
    }
    name = xmlParseName(ctxt);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"error parsing attribute name\n\x00" as *const u8 as *const i8,
        );

        return 0 as *const xmlChar;
    }
    /*
     * read the value
     */
    xmlSkipBlankChars(ctxt);
    if unsafe { *(*safe_ctxt.input).cur == '=' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        val = xmlParseAttValue(ctxt);
        safe_ctxt.instate = XML_PARSER_CONTENT
    } else {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_ATTRIBUTE_WITHOUT_VALUE,
            b"Specification mandates value for attribute %s\n\x00" as *const u8 as *const i8,
            name,
        );

        return 0 as *const xmlChar;
    }
    /*
     * Check that xml:lang conforms to the specification
     * No more registered as an error, just generate a warning now
     * since this was deprecated in XML second edition
     */
    if safe_ctxt.pedantic != 0
        && unsafe {
            xmlStrEqual_safe(
                name,
                b"xml:lang\x00" as *const u8 as *const i8 as *mut xmlChar,
            ) != 0
        }
    {
        if xmlCheckLanguageID(val) == 0 {
            xmlWarningMsg(
                ctxt,
                XML_WAR_LANG_VALUE,
                b"Malformed value for xml:lang : %s\n\x00" as *const u8 as *const i8,
                val,
                0 as *const xmlChar,
            );
        }
    }
    /*
     * Check that xml:space conforms to the specification
     */
    if unsafe {
        xmlStrEqual_safe(
            name,
            b"xml:space\x00" as *const u8 as *const i8 as *mut xmlChar,
        ) != 0
    } {
        if unsafe {
            xmlStrEqual_safe(
                val,
                b"default\x00" as *const u8 as *const i8 as *mut xmlChar,
            ) != 0
        } {
            unsafe { *safe_ctxt.space = 0 }
        } else if unsafe {
            xmlStrEqual_safe(
                val,
                b"preserve\x00" as *const u8 as *const i8 as *mut xmlChar,
            ) != 0
        } {
            unsafe { *safe_ctxt.space = 1 }
        } else {
            xmlWarningMsg(
                ctxt,
                XML_WAR_SPACE_VALUE,
                b"Invalid value \"%s\" for xml:space : \"default\" or \"preserve\" expected\n\x00"
                    as *const u8 as *const i8,
                val,
                0 as *const xmlChar,
            );
        }
    }
    unsafe { *value = val };
    return name;
}
/* *
* xmlParseStartTag:
* @ctxt:  an XML parser context
*
* parse a start of tag either for rule element or
* EmptyElement. In both case we don't parse the tag closing chars.
*
* [40] STag ::= '<' Name (S Attribute)* S? '>'
*
* [ WFC: Unique Att Spec ]
* No attribute name may appear more than once in the same start-tag or
* empty-element tag.
*
* [44] EmptyElemTag ::= '<' Name (S Attribute)* S? '/>'
*
* [ WFC: Unique Att Spec ]
* No attribute name may appear more than once in the same start-tag or
* empty-element tag.
*
* With namespace:
*
* [NS 8] STag ::= '<' QName (S Attribute)* S? '>'
*
* [NS 10] EmptyElement ::= '<' QName (S Attribute)* S? '/>'
*
* Returns the element name parsed
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseStartTag(mut ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };
    let mut current_block: u64;
    let name: *const xmlChar;
    let mut attname: *const xmlChar = 0 as *const xmlChar;
    let mut attvalue: *mut xmlChar = 0 as *mut xmlChar;
    let mut atts: *mut *const xmlChar = safe_ctxt.atts;
    let mut nbatts: i32 = 0;
    let mut maxatts: i32 = safe_ctxt.maxatts;
    let mut i: i32 = 0;

    if unsafe { *safe_input.cur != '<' as u8 } {
        return 0 as *const xmlChar;
    }
    safe_input.col += 1;
    safe_input.cur = unsafe { safe_input.cur.offset(1) };
    if unsafe { *safe_input.cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
    }

    name = xmlParseName(ctxt);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"xmlParseStartTag: invalid element name\n\x00" as *const u8 as *const i8,
        );

        return 0 as *const xmlChar;
    }
    /*
     * Now parse the attributes, it ends up with the ending
     *
     * (S Attribute)* S?
     */
    xmlSkipBlankChars(ctxt);
    if safe_ctxt.progressive == 0
        && unsafe { (safe_input.end.offset_from((*safe_ctxt.input).cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    while unsafe {
        *safe_input.cur != '>' as u8
            && (*safe_input.cur != '/' as u8 || *safe_input.cur.offset(1) != '>' as u8)
            && (0x9 <= *safe_input.cur as i32 && *safe_input.cur as i32 <= 0xa
                || *safe_input.cur as i32 == 0xd
                || 0x20 <= *safe_input.cur as i32)
    } && safe_ctxt.instate != XML_PARSER_EOF
    {
        let q: *const xmlChar = safe_input.cur;
        let cons: u64 = safe_input.consumed;
        attname = xmlParseAttribute(ctxt, &mut attvalue);
        if !attname.is_null() && !attvalue.is_null() {
            /*
             * [ WFC: Unique Att Spec ]
             * No attribute name may appear more than once in the same
             * start-tag or empty-element tag.
             */
            i = 0;
            loop {
                if i >= nbatts {
                    current_block = 1;
                    break;
                }
                if unsafe { xmlStrEqual_safe(*atts.offset(i as isize), attname) != 0 } {
                    xmlErrAttributeDup(ctxt, 0 as *const xmlChar, attname);
                    unsafe { xmlFree_safe(attvalue as *mut ()) };
                    current_block = 0;
                    break;
                } else {
                    i += 2
                }
            }
            match current_block {
                0 => {}
                _ =>
                /*
                 * Add the pair to atts
                 */
                {
                    if atts.is_null() {
                        maxatts = 22; /* allow for 10 attrs by default */
                        atts = unsafe {
                            xmlMalloc_safe(maxatts as u64 * size_of::<*mut xmlChar>() as u64)
                        } as *mut *const xmlChar;
                        if atts.is_null() {
                            xmlErrMemory(ctxt, 0 as *const i8);

                            if !attvalue.is_null() {
                                unsafe { xmlFree_safe(attvalue as *mut ()) };
                            }
                            current_block = 0;
                        } else {
                            safe_ctxt.atts = atts;
                            safe_ctxt.maxatts = maxatts;
                            current_block = 2;
                        }
                    } else if nbatts + 4 > maxatts {
                        let mut n: *mut *const xmlChar = 0 as *mut *const xmlChar;
                        maxatts *= 2;
                        n = unsafe {
                            xmlRealloc_safe(
                                atts as *mut (),
                                maxatts as u64 * size_of::<*const xmlChar>() as u64,
                            )
                        } as *mut *const xmlChar;
                        if n.is_null() {
                            xmlErrMemory(ctxt, 0 as *const i8);

                            if !attvalue.is_null() {
                                unsafe { xmlFree_safe(attvalue as *mut ()) };
                            }
                            current_block = 0;
                        } else {
                            atts = n;
                            safe_ctxt.atts = atts;
                            safe_ctxt.maxatts = maxatts;
                            current_block = 2;
                        }
                    } else {
                        current_block = 2;
                    }
                    match current_block {
                        0 => {}
                        _ => unsafe {
                            *atts.offset(nbatts as isize) = attname;
                            nbatts = nbatts + 1;
                            *atts.offset(nbatts as isize) = attvalue;
                            nbatts = nbatts + 1;
                            *atts.offset(nbatts as isize) = 0 as *const xmlChar;
                            *atts.offset((nbatts + 1) as isize) = 0 as *const xmlChar
                        },
                    }
                }
            }
        } else if !attvalue.is_null() {
            unsafe { xmlFree_safe(attvalue as *mut ()) };
        }
        if safe_ctxt.progressive == 0
            && unsafe { (safe_input.end.offset_from((*safe_ctxt.input).cur) as i64) < 250 }
        {
            xmlGROW(ctxt);
        }
        if unsafe {
            *safe_input.cur == '>' as u8
                || *safe_input.cur == '/' as u8 && *safe_input.cur.offset(1) == '>' as u8
        } {
            break;
        }
        if xmlSkipBlankChars(ctxt) == 0 {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_SPACE_REQUIRED,
                b"attributes construct error\n\x00" as *const u8 as *const i8,
            );
        }
        if cons == safe_input.consumed
            && q == safe_input.cur
            && attname.is_null()
            && attvalue.is_null()
        {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"xmlParseStartTag: problem parsing attributes\n\x00" as *const u8 as *const i8,
            );

            break;
        } else {
            if safe_ctxt.progressive == 0
                && unsafe {
                    safe_input.cur.offset_from(safe_input.base) as i64 > (2 * 250)
                        && (safe_input.end.offset_from(safe_input.cur) as i64) < (2 * 250)
                }
            {
                xmlSHRINK(ctxt);
            }
            if safe_ctxt.progressive == 0
                && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
            {
                xmlGROW(ctxt);
            }
        }
    }
    /*
     * SAX: Start of Element !
     */
    if !safe_ctxt.sax.is_null()
        && unsafe { (*safe_ctxt.sax).startElement.is_some() }
        && safe_ctxt.disableSAX == 0
    {
        if nbatts > 0 {
            unsafe {
                (*safe_ctxt.sax)
                    .startElement
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, name, atts
                );
            }
        } else {
            unsafe {
                (*safe_ctxt.sax)
                    .startElement
                    .expect("non-null function pointer")(
                    safe_ctxt.userData,
                    name,
                    0 as *mut *const xmlChar,
                );
            }
        }
    }
    if !atts.is_null() {
        /* Free only the content strings */
        i = 1;
        while i < nbatts {
            unsafe {
                if !(*atts.offset(i as isize)).is_null() {
                    xmlFree_safe(*atts.offset(i as isize) as *mut xmlChar as *mut ());
                }
            }
            i += 2
        }
    }
    return name;
}
/* *
* xmlParseEndTag1:
* @ctxt:  an XML parser context
* @line:  line of the start tag
* @nsNr:  number of namespaces on the start tag
*
* parse an end of tag
*
* [42] ETag ::= '</' Name S? '>'
*
* With namespace
*
* [NS 9] ETag ::= '</' QName S? '>'
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
fn xmlParseEndTag1(mut ctxt: xmlParserCtxtPtr, mut line: i32) {
    let mut name: *const xmlChar = 0 as *const xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };

    if safe_ctxt.progressive == 0
        && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    if unsafe { *safe_input.cur != '<' as u8 || *safe_input.cur.offset(1) != '/' as u8 } {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_LTSLASH_REQUIRED,
            b"xmlParseEndTag: \'</\' not found\n\x00" as *const u8 as *const i8,
        );
        return;
    }
    safe_input.cur = unsafe { safe_input.cur.offset(2) };
    safe_input.col += 2;
    if unsafe { *safe_input.cur as i32 == 0 } {
        unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
    }
    name = xmlParseNameAndCompare(ctxt, safe_ctxt.name);
    /*
     * We should definitely be at the ending "S? '>'" part
     */
    if safe_ctxt.progressive == 0
        && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    xmlSkipBlankChars(ctxt);
    if unsafe {
        !(0x9 <= *safe_input.cur as i32 && *safe_input.cur as i32 <= 0xa
            || *safe_input.cur as i32 == 0xd
            || 0x20 <= *safe_input.cur as i32)
            || *safe_input.cur != '>' as u8
    } {
        unsafe { xmlFatalErr(ctxt, XML_ERR_GT_REQUIRED, 0 as *const i8) };
    } else {
        safe_input.col += 1;
        safe_input.cur = unsafe { safe_input.cur.offset(1) };
        if unsafe { *safe_input.cur as i32 == 0 } {
            unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
        }
    }
    /*
     * [ WFC: Element Type Match ]
     * The Name in an element's end-tag must match the element type in the
     * start-tag.
     *
     */
    if name != 1 as *mut xmlChar {
        if name.is_null() {
            name = b"unparsable\x00" as *const u8 as *const i8 as *mut xmlChar
        }
        xmlFatalErrMsgStrIntStr(
            ctxt,
            XML_ERR_TAG_NAME_MISMATCH,
            b"Opening and ending tag mismatch: %s line %d and %s\n\x00" as *const u8 as *const i8,
            safe_ctxt.name,
            line,
            name,
        );
    }
    /*
     * SAX: End of Tag
     */
    if !safe_ctxt.sax.is_null()
        && unsafe { (*safe_ctxt.sax).endElement.is_some() }
        && safe_ctxt.disableSAX == 0
    {
        unsafe {
            (*safe_ctxt.sax)
                .endElement
                .expect("non-null function pointer")(safe_ctxt.userData, safe_ctxt.name)
        };
    }

    namePop(ctxt);
    spacePop(ctxt);
}
/* *
* xmlParseEndTag:
* @ctxt:  an XML parser context
*
* parse an end of tag
*
* [42] ETag ::= '</' Name S? '>'
*
* With namespace
*
* [NS 9] ETag ::= '</' QName S? '>'
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseEndTag(mut ctxt: xmlParserCtxtPtr) {
    xmlParseEndTag1(ctxt, 0);
}
/* LIBXML_SAX1_ENABLED */
/* ***********************************************************************
*									*
*		      SAX 2 specific operations				*
*									*
************************************************************************/
/*
* xmlGetNamespace:
* @ctxt:  an XML parser context
* @prefix:  the prefix to lookup
*
* Lookup the namespace name for the @prefix (which ca be NULL)
* The prefix must come from the @ctxt->dict dictionary
*
* Returns the namespace name or NULL if not bound
*/
fn xmlGetNamespace(mut ctxt: xmlParserCtxtPtr, prefix: *const xmlChar) -> *const xmlChar {
    let mut i: i32 = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if prefix == safe_ctxt.str_xml {
        return safe_ctxt.str_xml_ns;
    }
    i = safe_ctxt.nsNr - 2;
    while i >= 0 {
        if unsafe { *safe_ctxt.nsTab.offset(i as isize) } == prefix {
            if prefix.is_null() && unsafe { **safe_ctxt.nsTab.offset((i + 1) as isize) as i32 == 0 }
            {
                return 0 as *const xmlChar;
            }
            return unsafe { *safe_ctxt.nsTab.offset((i + 1) as isize) };
        }

        i -= 2
    }
    return 0 as *const xmlChar;
}
/* *
* xmlParseQName:
* @ctxt:  an XML parser context
* @prefix:  pointer to store the prefix part
*
* parse an XML Namespace QName
*
* [6]  QName  ::= (Prefix ':')? LocalPart
* [7]  Prefix  ::= NCName
* [8]  LocalPart  ::= NCName
*
* Returns the Name parsed or NULL
*/
fn xmlParseQName(mut ctxt: xmlParserCtxtPtr, mut prefix: *mut *const xmlChar) -> *const xmlChar {
    let mut l: *const xmlChar = 0 as *const xmlChar;
    let mut p: *const xmlChar = 0 as *const xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if safe_ctxt.progressive == 0
        && unsafe { ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250 }
    {
        xmlGROW(ctxt);
    }
    l = xmlParseNCName(ctxt);
    if l.is_null() {
        unsafe {
            if *(*safe_ctxt.input).cur == ':' as u8 {
                l = xmlParseName(ctxt);
                if !l.is_null() {
                    xmlNsErr(
                        ctxt,
                        XML_NS_ERR_QNAME,
                        b"Failed to parse QName \'%s\'\n\x00" as *const u8 as *const i8,
                        l,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                    );
                    *prefix = 0 as *const xmlChar;
                    return l;
                }
            }
        }
        return 0 as *const xmlChar;
    }
    if unsafe { *(*safe_ctxt.input).cur == ':' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        p = l;
        l = xmlParseNCName(ctxt);
        if l.is_null() {
            let mut tmp: *mut xmlChar = 0 as *mut xmlChar;
            if safe_ctxt.instate == XML_PARSER_EOF {
                return 0 as *const xmlChar;
            }
            xmlNsErr(
                ctxt,
                XML_NS_ERR_QNAME,
                b"Failed to parse QName \'%s:\'\n\x00" as *const u8 as *const i8,
                p,
                0 as *const xmlChar,
                0 as *const xmlChar,
            );
            l = xmlParseNmtoken(ctxt);
            if l.is_null() {
                if safe_ctxt.instate == XML_PARSER_EOF {
                    return 0 as *const xmlChar;
                }
                tmp = unsafe {
                    xmlBuildQName_safe(
                        b"\x00" as *const u8 as *const i8 as *mut xmlChar,
                        p,
                        0 as *mut xmlChar,
                        0,
                    )
                }
            } else {
                tmp = unsafe { xmlBuildQName_safe(l, p, 0 as *mut xmlChar, 0) };
                unsafe { xmlFree_safe(l as *mut i8 as *mut ()) };
            }
            p = unsafe { xmlDictLookup_safe(safe_ctxt.dict, tmp, -1) };
            if !tmp.is_null() {
                unsafe { xmlFree_safe(tmp as *mut ()) };
            }
            unsafe {
                *prefix = 0 as *const xmlChar;
            }
            return p;
        }
        if unsafe { *(*safe_ctxt.input).cur == ':' as u8 } {
            let mut tmp_0: *mut xmlChar = 0 as *mut xmlChar;
            xmlNsErr(
                ctxt,
                XML_NS_ERR_QNAME,
                b"Failed to parse QName \'%s:%s:\'\n\x00" as *const u8 as *const i8,
                p,
                l,
                0 as *const xmlChar,
            );
            unsafe { xmlNextChar_safe(ctxt) };
            tmp_0 = xmlParseName(ctxt) as *mut xmlChar;
            if !tmp_0.is_null() {
                tmp_0 = unsafe { xmlBuildQName_safe(tmp_0, l, 0 as *mut xmlChar, 0) };
                l = unsafe { xmlDictLookup_safe(safe_ctxt.dict, tmp_0, -1) };
                if !tmp_0.is_null() {
                    unsafe { xmlFree_safe(tmp_0 as *mut ()) };
                }
                unsafe {
                    *prefix = p;
                }
                return l;
            }
            if safe_ctxt.instate == XML_PARSER_EOF {
                return 0 as *const xmlChar;
            }
            tmp_0 = unsafe {
                xmlBuildQName_safe(
                    b"\x00" as *const u8 as *const i8 as *mut xmlChar,
                    l,
                    0 as *mut xmlChar,
                    0,
                )
            };
            l = unsafe { xmlDictLookup_safe(safe_ctxt.dict, tmp_0, -1) };
            if !tmp_0.is_null() {
                unsafe { xmlFree_safe(tmp_0 as *mut ()) };
            }
            unsafe {
                *prefix = p;
            }
            return l;
        }
        unsafe {
            *prefix = p;
        }
    } else {
        unsafe {
            *prefix = 0 as *const xmlChar;
        }
    }
    return l;
}
/* *
* xmlParseQNameAndCompare:
* @ctxt:  an XML parser context
* @name:  the localname
* @prefix:  the prefix, if any.
*
* parse an XML name and compares for match
* (specialized for endtag parsing)
*
* Returns NULL for an illegal name, (xmlChar*) 1 for success
* and the name for mismatch
*/
fn xmlParseQNameAndCompare(
    mut ctxt: xmlParserCtxtPtr,
    name: *const xmlChar,
    prefix: *const xmlChar,
) -> *const xmlChar {
    let mut cmp: *const xmlChar;
    let mut in_0: *const xmlChar;
    let ret: *const xmlChar;
    let mut prefix2: *const xmlChar = 0 as *const xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };
    if prefix.is_null() {
        return xmlParseNameAndCompare(ctxt, name);
    }
    if unsafe {
        safe_ctxt.progressive == 0
            && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    } {
        xmlGROW(ctxt);
    }
    in_0 = unsafe { (*safe_ctxt.input).cur };
    cmp = prefix;
    let mut safe_in_0 = unsafe { *in_0 };
    let mut safe_cmp = unsafe { *cmp };
    while safe_in_0 != 0 && safe_in_0 == safe_cmp {
        in_0 = unsafe { in_0.offset(1) };
        cmp = unsafe { cmp.offset(1) };
        safe_in_0 = unsafe { *in_0 };
        safe_cmp = unsafe { *cmp };
    }
    if safe_cmp == 0 && safe_in_0 == ':' as u8 {
        in_0 = unsafe { in_0.offset(1) };
        cmp = name;
        safe_in_0 = unsafe { *in_0 };
        safe_cmp = unsafe { *cmp };
        while safe_in_0 != 0 && safe_in_0 == safe_cmp {
            in_0 = unsafe { in_0.offset(1) };
            cmp = unsafe { cmp.offset(1) };
            safe_in_0 = unsafe { *in_0 };
            safe_cmp = unsafe { *cmp };
        }
        if safe_cmp == 0
            && (safe_in_0 == '>' as u8
                || (safe_in_0 == 0x20 || 0x9 <= safe_in_0 && safe_in_0 <= 0xa || safe_in_0 == 0xd))
        {
            /* success */
            safe_input.col = safe_input.col + unsafe { in_0.offset_from(safe_input.cur) } as i32;
            safe_input.cur = in_0;
            return 1 as *const xmlChar;
        }
    }

    /*
     * all strings coms from the dictionary, equality can be done directly
     */
    ret = xmlParseQName(ctxt, &mut prefix2);
    if ret == name && prefix == prefix2 {
        return 1 as *const xmlChar;
    }
    return ret;
}
/* *
* xmlParseAttValueInternal:
* @ctxt:  an XML parser context
* @len:  attribute len result
* @alloc:  whether the attribute was reallocated as a new string
* @normalize:  if 1 then further non-CDATA normalization must be done
*
* parse a value for an attribute.
* NOTE: if no normalization is needed, the routine will return pointers
*       directly from the data buffer.
*
* 3.3.3 Attribute-Value Normalization:
* Before the value of an attribute is passed to the application or
* checked for validity, the XML processor must normalize it as follows:
* - a character reference is processed by appending the referenced
*   character to the attribute value
* - an entity reference is processed by recursively processing the
*   replacement text of the entity
* - a whitespace character (#x20, #xD, #xA, #x9) is processed by
*   appending #x20 to the normalized value, except that only a single
*   #x20 is appended for a "#xD#xA" sequence that is part of an external
*   parsed entity or the literal entity value of an internal parsed entity
* - other characters are processed by appending them to the normalized value
* If the declared value is not CDATA, then the XML processor must further
* process the normalized attribute value by discarding any leading and
* trailing space (#x20) characters, and by replacing sequences of space
* (#x20) characters by a single space (#x20) character.
* All attributes for which no declaration has been read should be treated
* by a non-validating parser as if declared CDATA.
*
* Returns the AttValue parsed or NULL. The value has to be freed by the
*     caller if it was copied, this can be detected by val[*len] == 0.
*/
fn xmlParseAttValueInternal(
    mut ctxt: xmlParserCtxtPtr,
    mut len: *mut i32,
    mut alloc: *mut i32,
    normalize: i32,
) -> *mut xmlChar {
    let mut current_block: u64;
    let mut limit: xmlChar = 0 as xmlChar;
    let mut in_0: *const xmlChar = 0 as *const xmlChar;
    let mut start: *const xmlChar;
    let mut end: *const xmlChar;
    let mut last: *const xmlChar;
    let mut ret: *mut xmlChar = 0 as *mut xmlChar;
    let mut line: i32;
    let mut col: i32;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };
    if safe_ctxt.progressive == 0
        && unsafe { safe_input.end.offset_from(safe_input.cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }

    in_0 = safe_input.cur as *mut xmlChar;
    line = safe_input.line;
    col = safe_input.col;
    let mut safe_in_0 = unsafe { *in_0 };
    if safe_in_0 != '\"' as u8 && safe_in_0 != '\'' as u8 {
        unsafe { xmlFatalErr(ctxt, XML_ERR_ATTRIBUTE_NOT_STARTED, 0 as *const i8) };
        return 0 as *mut xmlChar;
    }

    safe_ctxt.instate = XML_PARSER_ATTRIBUTE_VALUE;
    /*
     * try to handle in this routine the most common case where no
     * allocation of a new string is required and where content is
     * pure ASCII.
     */

    limit = safe_in_0;
    in_0 = unsafe { in_0.offset(1) };
    col += 1;
    end = safe_input.end;

    start = in_0;
    if in_0 >= end {
        let oldbase: *const xmlChar = safe_input.base;
        if safe_ctxt.progressive == 0
            && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
        {
            xmlGROW(ctxt);
        }
        if safe_ctxt.instate == XML_PARSER_EOF {
            return 0 as *mut xmlChar;
        }

        if oldbase != safe_input.base {
            unsafe {
                let delta: ptrdiff_t = safe_input.base.offset_from(oldbase) as i64;
                start = start.offset(delta as isize);
                in_0 = in_0.offset(delta as isize)
            }
        }
        end = safe_input.end
    }
    safe_in_0 = unsafe { *in_0 };
    if normalize != 0 {
        /*
         * Skip any leading spaces
         */
        while in_0 < end
            && safe_in_0 != limit
            && (safe_in_0 == 0x20 || safe_in_0 == 0x9 || safe_in_0 == 0xa || safe_in_0 == 0xd)
        {
            if safe_in_0 == 0xa {
                line += 1;
                col = 1
            } else {
                col += 1
            }
            in_0 = unsafe { in_0.offset(1) };
            safe_in_0 = unsafe { *in_0 };
            start = in_0;
            if in_0 >= end {
                let oldbase: *const xmlChar = safe_input.base;
                if safe_ctxt.progressive == 0
                    && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) } < 250
                {
                    xmlGROW(ctxt);
                }
                if safe_ctxt.instate == XML_PARSER_EOF {
                    return 0 as *mut xmlChar;
                }

                if oldbase != safe_input.base {
                    unsafe {
                        let delta: ptrdiff_t = safe_input.base.offset_from(oldbase) as i64;
                        start = start.offset(delta as isize);
                        in_0 = in_0.offset(delta as isize)
                    }
                }
                end = safe_input.end;
                if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
                    && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
                {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ATTRIBUTE_NOT_FINISHED,
                        b"AttValue length too long\n\x00" as *const u8 as *const i8,
                    );
                    return 0 as *mut xmlChar;
                }
                safe_in_0 = unsafe { *in_0 };
            }
        }
        safe_in_0 = unsafe { *in_0 };
        while in_0 < end
            && safe_in_0 != limit
            && safe_in_0 >= 0x20
            && safe_in_0 <= 0x7f
            && safe_in_0 != '&' as u8
            && safe_in_0 != '<' as u8
        {
            col += 1;
            let in_0_old = unsafe { *in_0 };
            in_0 = unsafe { in_0.offset(1) };
            safe_in_0 = unsafe { *in_0 };
            if in_0_old == 0x20 && safe_in_0 == 0x20 {
                break;
            }
            if in_0 >= end {
                let oldbase: *const xmlChar = safe_input.base;
                if safe_ctxt.progressive == 0
                    && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
                {
                    xmlGROW(ctxt);
                }
                if safe_ctxt.instate == XML_PARSER_EOF {
                    return 0 as *mut xmlChar;
                }

                if oldbase != safe_input.base {
                    unsafe {
                        let delta: ptrdiff_t = safe_input.base.offset_from(oldbase) as i64;
                        start = start.offset(delta as isize);
                        in_0 = in_0.offset(delta as isize)
                    }
                }
                end = safe_input.end;
                if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
                    && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
                {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ATTRIBUTE_NOT_FINISHED,
                        b"AttValue length too long\n\x00" as *const u8 as *const i8,
                    );
                    return 0 as *mut xmlChar;
                }
                safe_in_0 = unsafe { *in_0 };
            }
        }
        last = in_0;
        /*
         * skip the trailing blanks
         */
        unsafe {
            while *last.offset(-1) == 0x20 && last > start {
                last = last.offset(-1)
            }
        }
        while in_0 < end
            && safe_in_0 != limit
            && (safe_in_0 == 0x20 || safe_in_0 == 0x9 || safe_in_0 == 0xa || safe_in_0 == 0xd)
        {
            if safe_in_0 == 0xa {
                line += 1;
                col = 1
            } else {
                col += 1
            }
            in_0 = unsafe { in_0.offset(1) };
            safe_in_0 = unsafe { *in_0 };
            if in_0 >= end {
                let oldbase: *const xmlChar = safe_input.base;
                if safe_ctxt.progressive == 0
                    && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
                {
                    xmlGROW(ctxt);
                }
                if safe_ctxt.instate == XML_PARSER_EOF {
                    return 0 as *mut xmlChar;
                }

                if oldbase != safe_input.base {
                    unsafe {
                        let delta: ptrdiff_t = safe_input.base.offset_from(oldbase) as i64;
                        start = start.offset(delta as isize);
                        in_0 = in_0.offset(delta as isize);
                        last = last.offset(delta as isize)
                    }
                }
                end = safe_input.end;
                if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
                    && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
                {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ATTRIBUTE_NOT_FINISHED,
                        b"AttValue length too long\n\x00" as *const u8 as *const i8,
                    );
                    return 0 as *mut xmlChar;
                }
                safe_in_0 = unsafe { *in_0 };
            }
        }

        if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
            && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
        {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_ATTRIBUTE_NOT_FINISHED,
                b"AttValue length too long\n\x00" as *const u8 as *const i8,
            );
            return 0 as *mut xmlChar;
        }
        safe_in_0 = unsafe { *in_0 };
        if safe_in_0 != limit {
            current_block = 0;
        } else {
            current_block = 1;
        }
    } else {
        while in_0 < end
            && safe_in_0 != limit
            && safe_in_0 >= 0x20
            && safe_in_0 <= 0x7f
            && safe_in_0 != '&' as u8
            && safe_in_0 != '<' as u8
        {
            in_0 = unsafe { in_0.offset(1) };
            safe_in_0 = unsafe { *in_0 };
            col += 1;
            if in_0 >= end {
                let oldbase: *const xmlChar = safe_input.base;
                if safe_ctxt.progressive == 0
                    && unsafe { (safe_input.end.offset_from(safe_input.cur) as i64) < 250 }
                {
                    xmlGROW(ctxt);
                }
                if safe_ctxt.instate == XML_PARSER_EOF {
                    return 0 as *mut xmlChar;
                }

                if oldbase != safe_input.base {
                    unsafe {
                        let delta: ptrdiff_t = safe_input.base.offset_from(oldbase) as i64;
                        start = start.offset(delta as isize);
                        in_0 = in_0.offset(delta as isize)
                    }
                }
                end = safe_input.end;
                if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
                    && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
                {
                    xmlFatalErrMsg(
                        ctxt,
                        XML_ERR_ATTRIBUTE_NOT_FINISHED,
                        b"AttValue length too long\n\x00" as *const u8 as *const i8,
                    );
                    return 0 as *mut xmlChar;
                }
                safe_in_0 = unsafe { *in_0 };
            }
        }
        last = in_0;

        if unsafe { in_0.offset_from(start) } as u64 > XML_MAX_TEXT_LENGTH
            && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
        {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_ATTRIBUTE_NOT_FINISHED,
                b"AttValue length too long\n\x00" as *const u8 as *const i8,
            );
            return 0 as *mut xmlChar;
        }

        safe_in_0 = unsafe { *in_0 };
        if safe_in_0 != limit {
            current_block = 0;
        } else {
            current_block = 1;
        }
    }
    match current_block {
        0 => {
            if !alloc.is_null() {
                unsafe { *alloc = 1 }
            }
            return xmlParseAttValueComplex(ctxt, len, normalize);
        }
        _ => {
            in_0 = unsafe { in_0.offset(1) };
            col += 1;
            if !len.is_null() {
                unsafe {
                    *len = last.offset_from(start) as i32;
                }
                ret = start as *mut xmlChar
            } else {
                if !alloc.is_null() {
                    unsafe { *alloc = 1 }
                }
                ret = unsafe { xmlStrndup_safe(start, last.offset_from(start) as i32) };
            }

            safe_input.cur = in_0;
            safe_input.line = line;
            safe_input.col = col;
            if !alloc.is_null() {
                unsafe { *alloc = 0 }
            }

            return ret;
        }
    };
}
/* *
* xmlParseAttribute2:
* @ctxt:  an XML parser context
* @pref:  the element prefix
* @elem:  the element name
* @prefix:  a xmlChar ** used to store the value of the attribute prefix
* @value:  a xmlChar ** used to store the value of the attribute
* @len:  an int * to save the length of the attribute
* @alloc:  an int * to indicate if the attribute was allocated
*
* parse an attribute in the new SAX2 framework.
*
* Returns the attribute name, and the value in *value, .
*/
fn xmlParseAttribute2(
    mut ctxt: xmlParserCtxtPtr,
    pref: *const xmlChar,
    elem: *const xmlChar,
    mut prefix: *mut *const xmlChar,
    mut value: *mut *mut xmlChar,
    len: *mut i32,
    mut alloc: *mut i32,
) -> *const xmlChar {
    let name: *const xmlChar;
    let mut val: *mut xmlChar = 0 as *mut xmlChar;
    let mut internal_val: *mut xmlChar = 0 as *mut xmlChar;
    let mut normalize: i32 = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    unsafe { *value = 0 as *mut xmlChar };
    if safe_ctxt.progressive == 0
        && unsafe { ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) } < 250
    {
        xmlGROW(ctxt);
    }
    name = xmlParseQName(ctxt, prefix);
    if name.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"error parsing attribute name\n\x00" as *const u8 as *const i8,
        );

        return 0 as *const xmlChar;
    }
    /*
     * get the type if needed
     */
    if !safe_ctxt.attsSpecial.is_null() {
        let type_0: i32;
        type_0 = unsafe { xmlHashQLookup2_safe(safe_ctxt.attsSpecial, pref, elem, *prefix, name) }
            as ptrdiff_t as i32;
        if type_0 != 0 {
            normalize = 1
        }
    }
    /*
     * read the value
     */
    xmlSkipBlankChars(ctxt);
    if unsafe { *(*safe_ctxt.input).cur == '=' as u8 } {
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        val = xmlParseAttValueInternal(ctxt, len, alloc, normalize);
        if normalize != 0 {
            /*
             * Sometimes a second normalisation pass for spaces is needed
             * but that only happens if charrefs or entities references
             * have been used in the attribute value, i.e. the attribute
             * value have been extracted in an allocated string already.
             */
            if unsafe { *alloc != 0 } {
                let val2: *const xmlChar;
                val2 = xmlAttrNormalizeSpace2(ctxt, val, len);
                if !val2.is_null() && val2 != val {
                    unsafe { xmlFree_safe(val as *mut ()) };
                    val = val2 as *mut xmlChar
                }
            }
        }
        safe_ctxt.instate = XML_PARSER_CONTENT
    } else {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_ATTRIBUTE_WITHOUT_VALUE,
            b"Specification mandates value for attribute %s\n\x00" as *const u8 as *const i8,
            name,
        );

        return 0 as *const xmlChar;
    }
    if unsafe { *prefix } == safe_ctxt.str_xml {
        /*
         * Check that xml:lang conforms to the specification
         * No more registered as an error, just generate a warning now
         * since this was deprecated in XML second edition
         */
        if safe_ctxt.pedantic != 0
            && unsafe {
                xmlStrEqual_safe(name, b"lang\x00" as *const u8 as *const i8 as *mut xmlChar)
            } != 0
        {
            internal_val = unsafe { xmlStrndup_safe(val, *len) };
            if xmlCheckLanguageID(internal_val) == 0 {
                xmlWarningMsg(
                    ctxt,
                    XML_WAR_LANG_VALUE,
                    b"Malformed value for xml:lang : %s\n\x00" as *const u8 as *const i8,
                    internal_val,
                    0 as *const xmlChar,
                );
            }
        }
        /*
         * Check that xml:space conforms to the specification
         */
        if unsafe { xmlStrEqual_safe(name, b"space\x00" as *const u8 as *const i8 as *mut xmlChar) }
            != 0
        {
            internal_val = unsafe { xmlStrndup_safe(val, *len) };
            if unsafe {
                xmlStrEqual_safe(
                    internal_val,
                    b"default\x00" as *const u8 as *const i8 as *mut xmlChar,
                )
            } != 0
            {
                unsafe { *safe_ctxt.space = 0 }
            } else if unsafe {
                xmlStrEqual_safe(
                    internal_val,
                    b"preserve\x00" as *const u8 as *const i8 as *mut xmlChar,
                )
            } != 0
            {
                unsafe { *safe_ctxt.space = 1 }
            } else {
                xmlWarningMsg(ctxt, XML_WAR_SPACE_VALUE,
              b"Invalid value \"%s\" for xml:space : \"default\" or \"preserve\" expected\n\x00"
                  as *const u8 as *const i8,
              internal_val, 0 as *const xmlChar);
            }
        }
        if !internal_val.is_null() {
            unsafe { xmlFree_safe(internal_val as *mut ()) };
        }
    }
    unsafe {
        *value = val;
    }
    return name;
}
/* *
* xmlParseStartTag2:
* @ctxt:  an XML parser context
*
* parse a start of tag either for rule element or
* EmptyElement. In both case we don't parse the tag closing chars.
* This routine is called when running SAX2 parsing
*
* [40] STag ::= '<' Name (S Attribute)* S? '>'
*
* [ WFC: Unique Att Spec ]
* No attribute name may appear more than once in the same start-tag or
* empty-element tag.
*
* [44] EmptyElemTag ::= '<' Name (S Attribute)* S? '/>'
*
* [ WFC: Unique Att Spec ]
* No attribute name may appear more than once in the same start-tag or
* empty-element tag.
*
* With namespace:
*
* [NS 8] STag ::= '<' QName (S Attribute)* S? '>'
*
* [NS 10] EmptyElement ::= '<' QName (S Attribute)* S? '/>'
*
* Returns the element name parsed
*/
fn xmlParseStartTag2(
    mut ctxt: xmlParserCtxtPtr,
    mut pref: *mut *const xmlChar,
    mut URI: *mut *const xmlChar,
    mut tlen: *mut i32,
) -> *const xmlChar {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut current_block: u64;
    let mut localname: *const xmlChar;
    let mut prefix: *const xmlChar = 0 as *const xmlChar;
    let mut attname: *const xmlChar;
    let mut aprefix: *const xmlChar = 0 as *const xmlChar;
    let mut nsname: *const xmlChar;
    let mut attvalue: *mut xmlChar = 0 as *mut xmlChar;
    let mut atts: *mut *const xmlChar = safe_ctxt.atts;
    let mut maxatts: i32 = safe_ctxt.maxatts;
    let mut nratts: i32;
    let mut nbatts: i32;
    let mut nbdef: i32;
    let mut inputid: i32;
    let mut i: i32;
    let mut j: i32;
    let mut nbNs: i32;
    let mut attval: i32;
    let cur: u64;
    let nsNr: i32 = safe_ctxt.nsNr;
    let mut safe_input = unsafe { *safe_ctxt.input };
    if unsafe { *safe_input.cur } != '<' as u8 {
        return 0 as *const xmlChar;
    }
    safe_input.col += 1;
    unsafe {
        (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(1);
        if *safe_input.cur as i32 == 0 {
            xmlParserInputGrow_safe(safe_ctxt.input, 250);
        }
    }
    /*
     * NOTE: it is crucial with the SAX2 API to never call SHRINK beyond that
     *       point since the attribute values may be stored as pointers to
     *       the buffer and calling SHRINK would destroy them !
     *       The Shrinking is only possible once the full set of attribute
     *       callbacks have been done.
     */
    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).cur.offset_from(safe_input.base) } as i64 > 2 * 250
        && (unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) } as i64) < 2 * 250
    {
        xmlSHRINK(ctxt);
    }
    cur = unsafe { (*safe_ctxt.input).cur.offset_from(safe_input.base) } as u64;
    inputid = safe_input.id;
    nbatts = 0;
    nratts = 0;
    nbdef = 0;
    nbNs = 0;
    attval = 0;
    /* Forget any namespaces added during an earlier parse of this element. */
    safe_ctxt.nsNr = nsNr;

    localname = xmlParseQName(ctxt, &mut prefix);
    if localname.is_null() {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_NAME_REQUIRED,
            b"StartTag: invalid element name\n\x00" as *const u8 as *const i8,
        );
        return 0 as *const xmlChar;
    }
    unsafe {
        *tlen = ((*safe_ctxt.input).cur.offset_from(safe_input.base) as u64 - cur) as i32;
    }
    /*
     * Now parse the attributes, it ends up with the ending
     *
     * (S Attribute)* S?
     */
    xmlSkipBlankChars(ctxt);
    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }
    loop {
        if unsafe {
            !(*(*safe_ctxt.input).cur != '>' as u8
                && (*(*safe_ctxt.input).cur != '/' as u8
                    || *(*safe_ctxt.input).cur.offset(1) != '>' as u8)
                && (0x9 <= *(*safe_ctxt.input).cur as i32 && *(*safe_ctxt.input).cur as i32 <= 0xa
                    || *(*safe_ctxt.input).cur as i32 == 0xd
                    || 0x20 <= *(*safe_ctxt.input).cur as i32)
                && safe_ctxt.instate != XML_PARSER_EOF)
        } {
            current_block = 1;
            break;
        }
        let q: *const xmlChar = safe_input.cur;
        let cons: u64 = safe_input.consumed;
        let mut len: i32 = -1;
        let mut alloc: i32 = 0;
        attname = xmlParseAttribute2(
            ctxt,
            prefix,
            localname,
            &mut aprefix,
            &mut attvalue,
            &mut len,
            &mut alloc,
        );
        if !(attname.is_null() || attvalue.is_null()) {
            if len < 0 {
                len = unsafe { xmlStrlen_safe(attvalue) }
            }
            if attname == safe_ctxt.str_xmlns && aprefix.is_null() {
                let mut URL: *const xmlChar =
                    unsafe { xmlDictLookup_safe(safe_ctxt.dict, attvalue, len) };
                let mut uri: xmlURIPtr = 0 as *mut xmlURI;
                if URL.is_null() {
                    xmlErrMemory(
                        ctxt,
                        b"dictionary allocation failure\x00" as *const u8 as *const i8,
                    );

                    if !attvalue.is_null() && alloc != 0 {
                        unsafe { xmlFree_safe(attvalue as *mut ()) };
                    }
                    localname = 0 as *const xmlChar;
                    current_block = 2;
                    break;
                } else {
                    if unsafe { *URL } != 0 {
                        uri = unsafe { xmlParseURI_safe(URL as *const i8) };
                        if uri.is_null() {
                            xmlNsErr(
                                ctxt,
                                XML_WAR_NS_URI,
                                b"xmlns: \'%s\' is not a valid URI\n\x00" as *const u8 as *const i8,
                                URL,
                                0 as *const xmlChar,
                                0 as *const xmlChar,
                            );
                        } else {
                            if unsafe { (*uri).scheme.is_null() } {
                                xmlNsWarn(
                                    ctxt,
                                    XML_WAR_NS_URI_RELATIVE,
                                    b"xmlns: URI %s is not absolute\n\x00" as *const u8
                                        as *const i8,
                                    URL,
                                    0 as *const xmlChar,
                                    0 as *const xmlChar,
                                );
                            }
                            unsafe { xmlFreeURI_safe(uri) };
                        }
                        if URL == safe_ctxt.str_xml_ns {
                            if attname != safe_ctxt.str_xml {
                                xmlNsErr(
                                    ctxt,
                                    XML_NS_ERR_XML_NAMESPACE,
                                    b"xml namespace URI cannot be the default namespace\n\x00"
                                        as *const u8
                                        as *const i8,
                                    0 as *const xmlChar,
                                    0 as *const xmlChar,
                                    0 as *const xmlChar,
                                );
                            }
                            current_block = 0;
                        } else if len == 29
                            && unsafe {
                                xmlStrEqual_safe(
                                    URL,
                                    b"http://www.w3.org/2000/xmlns/\x00" as *const u8 as *const i8
                                        as *mut xmlChar,
                                )
                            } != 0
                        {
                            xmlNsErr(
                                ctxt,
                                XML_NS_ERR_XML_NAMESPACE,
                                b"reuse of the xmlns namespace name is forbidden\n\x00" as *const u8
                                    as *const i8,
                                0 as *const xmlChar,
                                0 as *const xmlChar,
                                0 as *const xmlChar,
                            );
                            current_block = 0;
                        } else {
                            current_block = 3;
                        }
                    } else {
                        current_block = 3;
                    }
                    match current_block {
                        0 => {}
                        _ => {
                            /*
                             * check that it's not a defined namespace
                             */
                            j = 1;
                            while j <= nbNs {
                                if unsafe {
                                    (*safe_ctxt.nsTab.offset((safe_ctxt.nsNr - 2 * j) as isize))
                                        .is_null()
                                } {
                                    break;
                                }
                                j += 1
                            }
                            if j <= nbNs {
                                xmlErrAttributeDup(ctxt, 0 as *const xmlChar, attname);
                            } else if nsPush(ctxt, 0 as *const xmlChar, URL) > 0 {
                                nbNs += 1
                            }
                        }
                    }
                }
            } else if aprefix == safe_ctxt.str_xmlns {
                let URL: *const xmlChar =
                    unsafe { xmlDictLookup_safe(safe_ctxt.dict, attvalue, len) };
                let mut uri: xmlURIPtr = 0 as *mut xmlURI;
                if attname == safe_ctxt.str_xml {
                    if URL != safe_ctxt.str_xml_ns {
                        xmlNsErr(
                            ctxt,
                            XML_NS_ERR_XML_NAMESPACE,
                            b"xml namespace prefix mapped to wrong URI\n\x00" as *const u8
                                as *const i8,
                            0 as *const xmlChar,
                            0 as *const xmlChar,
                            0 as *const xmlChar,
                        );
                    }
                } else if URL == safe_ctxt.str_xml_ns {
                    if attname != safe_ctxt.str_xml {
                        xmlNsErr(
                            ctxt,
                            XML_NS_ERR_XML_NAMESPACE,
                            b"xml namespace URI mapped to wrong prefix\n\x00" as *const u8
                                as *const i8,
                            0 as *const xmlChar,
                            0 as *const xmlChar,
                            0 as *const xmlChar,
                        );
                    }
                } else if attname == safe_ctxt.str_xmlns {
                    xmlNsErr(
                        ctxt,
                        XML_NS_ERR_XML_NAMESPACE,
                        b"redefinition of the xmlns prefix is forbidden\n\x00" as *const u8
                            as *const i8,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                    );
                } else if len == 29
                    && unsafe {
                        xmlStrEqual_safe(
                            URL,
                            b"http://www.w3.org/2000/xmlns/\x00" as *const u8 as *const i8
                                as *mut xmlChar,
                        )
                    } != 0
                {
                    xmlNsErr(
                        ctxt,
                        XML_NS_ERR_XML_NAMESPACE,
                        b"reuse of the xmlns namespace name is forbidden\n\x00" as *const u8
                            as *const i8,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                    );
                } else if URL.is_null() || unsafe { *URL.offset(0 as isize) as i32 == 0 } {
                    xmlNsErr(
                        ctxt,
                        XML_NS_ERR_XML_NAMESPACE,
                        b"xmlns:%s: Empty XML namespace is not allowed\n\x00" as *const u8
                            as *const i8,
                        attname,
                        0 as *const xmlChar,
                        0 as *const xmlChar,
                    );
                } else {
                    uri = unsafe { xmlParseURI_safe(URL as *const i8) };
                    if uri.is_null() {
                        xmlNsErr(
                            ctxt,
                            XML_WAR_NS_URI,
                            b"xmlns:%s: \'%s\' is not a valid URI\n\x00" as *const u8 as *const i8,
                            attname,
                            URL,
                            0 as *const xmlChar,
                        );
                    } else {
                        if safe_ctxt.pedantic != 0 && unsafe { (*uri).scheme.is_null() } {
                            xmlNsWarn(
                                ctxt,
                                XML_WAR_NS_URI_RELATIVE,
                                b"xmlns:%s: URI %s is not absolute\n\x00" as *const u8 as *const i8,
                                attname,
                                URL,
                                0 as *const xmlChar,
                            );
                        }
                        unsafe { xmlFreeURI_safe(uri) };
                    }
                    /*
                     * check that it's not a defined namespace
                     */
                    j = 1;
                    while j <= nbNs {
                        if unsafe {
                            *safe_ctxt.nsTab.offset((safe_ctxt.nsNr - 2 * j) as isize) == attname
                        } {
                            break;
                        }
                        j += 1
                    }
                    if j <= nbNs {
                        xmlErrAttributeDup(ctxt, aprefix, attname);
                    } else if nsPush(ctxt, attname, URL) > 0 {
                        nbNs += 1
                    }
                }
            } else {
                /*
                 * Add the pair to atts
                 */
                if atts.is_null() || nbatts + 5 > maxatts {
                    if xmlCtxtGrowAttrs(ctxt, nbatts + 5) < 0 {
                        current_block = 0;
                    } else {
                        maxatts = safe_ctxt.maxatts;
                        atts = safe_ctxt.atts;
                        current_block = 4;
                    }
                } else {
                    current_block = 4;
                }
                match current_block {
                    0 => {}
                    _ => {
                        unsafe {
                            *safe_ctxt.attallocs.offset(nratts as isize) = alloc;
                            nratts = nratts + 1;
                            *atts.offset(nbatts as isize) = attname;
                            nbatts = nbatts + 1;
                            *atts.offset(nbatts as isize) = aprefix;
                            nbatts = nbatts + 1;
                            /*
                             * The namespace URI field is used temporarily to point at the
                             * base of the current input buffer for non-alloced attributes.
                             * When the input buffer is reallocated, all the pointers become
                             * invalid, but they can be reconstructed later.
                             */
                            if alloc != 0 {
                                *atts.offset(nbatts as isize) = 0 as *const xmlChar;
                                nbatts = nbatts + 1;
                            } else {
                                *atts.offset(nbatts as isize) = (*safe_ctxt.input).base;
                                nbatts = nbatts + 1;
                            }
                            *atts.offset(nbatts as isize) = attvalue;
                            nbatts = nbatts + 1;
                            attvalue = attvalue.offset(len as isize);
                            *atts.offset(nbatts as isize) = attvalue;
                            nbatts = nbatts + 1;
                        }
                        /*
                         * tag if some deallocation is needed
                         */
                        if alloc != 0 {
                            attval = 1
                        }
                        attvalue = 0 as *mut xmlChar
                    }
                }
                /* moved into atts */
            }
        }
        /*
         * Do not keep a namespace definition node
         */
        if !attvalue.is_null() && alloc != 0 {
            unsafe { xmlFree_safe(attvalue as *mut ()) };
            attvalue = 0 as *mut xmlChar
        }
        if unsafe {
            safe_ctxt.progressive == 0
                && ((*safe_ctxt.input).end.offset_from(safe_input.cur) as i64) < 250
        } {
            xmlGROW(ctxt);
        }
        if safe_ctxt.instate == XML_PARSER_EOF {
            current_block = 1;
            break;
        }
        if unsafe {
            *(*safe_ctxt.input).cur == '>' as u8
                || *(*safe_ctxt.input).cur == '/' as u8
                    && *(*safe_ctxt.input).cur.offset(1) == '>' as u8
        } {
            current_block = 1;
            break;
        }
        if xmlSkipBlankChars(ctxt) == 0 {
            xmlFatalErrMsg(
                ctxt,
                XML_ERR_SPACE_REQUIRED,
                b"attributes construct error\n\x00" as *const u8 as *const i8,
            );
            current_block = 1;
            break;
        } else if cons == safe_input.consumed
            && q == safe_input.cur
            && attname.is_null()
            && attvalue.is_null()
        {
            unsafe {
                xmlFatalErr(
                    ctxt,
                    XML_ERR_INTERNAL_ERROR,
                    b"xmlParseStartTag: problem parsing attributes\n\x00" as *const u8 as *const i8,
                )
            };
            current_block = 1;
            break;
        } else if unsafe {
            safe_ctxt.progressive == 0
                && ((*safe_ctxt.input).end.offset_from(safe_input.cur) as i64) < 250
        } {
            xmlGROW(ctxt);
        }
    }
    match current_block {
        1 => {
            if safe_input.id != inputid {
                unsafe {
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_INTERNAL_ERROR,
                        b"Unexpected change of input\n\x00" as *const u8 as *const i8,
                    )
                };
                localname = 0 as *const xmlChar
            } else {
                /* Reconstruct attribute value pointers. */
                i = 0;
                j = 0;
                while j < nratts {
                    unsafe {
                        if !(*atts.offset((i + 2) as isize)).is_null() {
                            /*
                             * Arithmetic on dangling pointers is technically undefined
                             * behavior, but well...
                             */
                            let mut offset: ptrdiff_t = (*safe_ctxt.input)
                                .base
                                .offset_from(*atts.offset((i + 2) as isize))
                                as i64; /* Reset repurposed namespace URI */
                            *atts.offset((i + 2) as isize) = 0 as *const xmlChar; /* value */
                            let ref mut atts_3 = *atts.offset((i + 3) as isize);
                            *atts_3 = (*atts_3).offset(offset as isize);
                            let ref mut atts_4 = *atts.offset((i + 4) as isize);
                            *atts_4 = (*atts_4).offset(offset as isize)
                        }
                    }
                    i += 5;
                    j += 1
                }
                /*
                 * The attributes defaulting
                 */
                if !safe_ctxt.attsDefault.is_null() {
                    let mut defaults: xmlDefAttrsPtr = 0 as *mut xmlDefAttrs;
                    defaults =
                        unsafe { xmlHashLookup2_safe(safe_ctxt.attsDefault, localname, prefix) }
                            as xmlDefAttrsPtr;
                    if !defaults.is_null() {
                        i = 0;
                        loop {
                            unsafe {
                                if !(i < (*defaults).nbAttrs) {
                                    current_block = 5;
                                    break;
                                }
                                attname = *(*defaults).values.as_mut_ptr().offset((5 * i) as isize);
                                aprefix =
                                    *(*defaults).values.as_mut_ptr().offset((5 * i + 1) as isize);
                            }

                            /*
                             * special work for namespaces defaulted defs
                             */
                            if attname == safe_ctxt.str_xmlns && aprefix.is_null() {
                                /*
                                 * check that it's not a defined namespace
                                 */
                                j = 1;
                                while j <= nbNs {
                                    if unsafe {
                                        (*safe_ctxt.nsTab.offset((safe_ctxt.nsNr - 2 * j) as isize))
                                            .is_null()
                                    } {
                                        break;
                                    }
                                    j += 1
                                }
                                if !(j <= nbNs) {
                                    nsname = xmlGetNamespace(ctxt, 0 as *const xmlChar);
                                    if unsafe {
                                        nsname
                                            != *(*defaults)
                                                .values
                                                .as_mut_ptr()
                                                .offset((5 * i + 2) as isize)
                                    } {
                                        if unsafe {
                                            nsPush(
                                                ctxt,
                                                0 as *const xmlChar,
                                                *(*defaults)
                                                    .values
                                                    .as_mut_ptr()
                                                    .offset((5 * i + 2) as isize),
                                            ) > 0
                                        } {
                                            nbNs += 1
                                        }
                                    }
                                }
                            } else if aprefix == safe_ctxt.str_xmlns {
                                /*
                                 * check that it's not a defined namespace
                                 */
                                j = 1;
                                while j <= nbNs {
                                    if unsafe {
                                        *safe_ctxt.nsTab.offset((safe_ctxt.nsNr - 2 * j) as isize)
                                            == attname
                                    } {
                                        break;
                                    }
                                    j += 1
                                }
                                if !(j <= nbNs) {
                                    nsname = xmlGetNamespace(ctxt, attname);
                                    if unsafe {
                                        nsname != *(*defaults).values.as_mut_ptr().offset(2)
                                    } {
                                        if unsafe {
                                            nsPush(
                                                ctxt,
                                                attname,
                                                *(*defaults)
                                                    .values
                                                    .as_mut_ptr()
                                                    .offset((5 * i + 2) as isize),
                                            ) > 0
                                        } {
                                            nbNs += 1
                                        }
                                    }
                                }
                            } else {
                                /*
                                 * check that it's not a defined attribute
                                 */
                                j = 0;
                                while j < nbatts {
                                    unsafe {
                                        if attname == *atts.offset(j as isize)
                                            && aprefix == *atts.offset((j + 1) as isize)
                                        {
                                            break;
                                        }
                                    }
                                    j += 5
                                }
                                if !(j < nbatts) {
                                    if atts.is_null() || nbatts + 5 > maxatts {
                                        if xmlCtxtGrowAttrs(ctxt, nbatts + 5) < 0 {
                                            localname = 0 as *const xmlChar;
                                            current_block = 2;
                                            break;
                                        } else {
                                            maxatts = safe_ctxt.maxatts;
                                            atts = safe_ctxt.atts
                                        }
                                    }
                                    unsafe {
                                        *atts.offset(nbatts as isize) = attname;
                                        nbatts = nbatts + 1;
                                        *atts.offset(nbatts as isize) = aprefix;
                                        nbatts = nbatts + 1;
                                        if aprefix.is_null() {
                                            *atts.offset(nbatts as isize) = 0 as *const xmlChar;
                                            nbatts = nbatts + 1;
                                        } else {
                                            *atts.offset(nbatts as isize) =
                                                xmlGetNamespace(ctxt, aprefix);
                                            nbatts = nbatts + 1;
                                        }
                                        *atts.offset(nbatts as isize) = *(*defaults)
                                            .values
                                            .as_mut_ptr()
                                            .offset((5 * i + 2) as isize);
                                        nbatts = nbatts + 1;
                                        *atts.offset(nbatts as isize) = *(*defaults)
                                            .values
                                            .as_mut_ptr()
                                            .offset((5 * i + 3) as isize);
                                        nbatts = nbatts + 1;
                                    }
                                    if safe_ctxt.standalone == 1
                                        && unsafe {
                                            !(*(*defaults)
                                                .values
                                                .as_mut_ptr()
                                                .offset((5 * i + 4) as isize))
                                            .is_null()
                                        }
                                    {
                                        xmlValidityError(ctxt,
                                     XML_DTD_STANDALONE_DEFAULTED,
                                     b"standalone: attribute %s on %s defaulted from external subset\n\x00"
                                         as *const u8 as
                                         *const i8,
                                     attname, localname);
                                    }
                                    nbdef += 1
                                }
                            }
                            i += 1
                        }
                    } else {
                        current_block = 5;
                    }
                } else {
                    current_block = 5;
                }
                match current_block {
                    2 => {}
                    _ => {
                        /*
                         * The attributes checkings
                         */
                        i = 0;
                        while i < nbatts {
                            /*
                             * The default namespace does not apply to attribute names.
                             */
                            if unsafe { !(*atts.offset((i + 1) as isize)).is_null() } {
                                unsafe {
                                    nsname = xmlGetNamespace(ctxt, *atts.offset((i + 1) as isize));
                                    if nsname.is_null() {
                                        xmlNsErr(
                                            ctxt,
                                            XML_NS_ERR_UNDEFINED_NAMESPACE,
                                            b"Namespace prefix %s for %s on %s is not defined\n\x00"
                                                as *const u8
                                                as *const i8,
                                            *atts.offset((i + 1) as isize),
                                            *atts.offset(i as isize),
                                            localname,
                                        );
                                    }
                                    *atts.offset((i + 2) as isize) = nsname;
                                }
                            } else {
                                nsname = 0 as *const xmlChar
                            }
                            /*
                             * [ WFC: Unique Att Spec ]
                             * No attribute name may appear more than once in the same
                             * start-tag or empty-element tag.
                             * As extended by the Namespace in XML REC.
                             */
                            j = 0;
                            while j < i {
                                unsafe {
                                    if *atts.offset(i as isize) == *atts.offset(j as isize) {
                                        if *atts.offset((i + 1) as isize)
                                            == *atts.offset((j + 1) as isize)
                                        {
                                            xmlErrAttributeDup(
                                                ctxt,
                                                *atts.offset((i + 1) as isize),
                                                *atts.offset(i as isize),
                                            );
                                            break;
                                        } else if !nsname.is_null()
                                            && *atts.offset((j + 2) as isize) == nsname
                                        {
                                            xmlNsErr(
                                                ctxt,
                                                XML_NS_ERR_ATTRIBUTE_REDEFINED,
                                                b"Namespaced Attribute %s in \'%s\' redefined\n\x00"
                                                    as *const u8
                                                    as *const i8,
                                                *atts.offset(i as isize),
                                                nsname,
                                                0 as *const xmlChar,
                                            );
                                            break;
                                        }
                                    }
                                }
                                j += 5
                            }
                            i += 5
                        }
                        nsname = xmlGetNamespace(ctxt, prefix);
                        if !prefix.is_null() && nsname.is_null() {
                            xmlNsErr(
                                ctxt,
                                XML_NS_ERR_UNDEFINED_NAMESPACE,
                                b"Namespace prefix %s on %s is not defined\n\x00" as *const u8
                                    as *const i8,
                                prefix,
                                localname,
                                0 as *const xmlChar,
                            );
                        }
                        unsafe {
                            *pref = prefix;
                            *URI = nsname;
                            /*
                             * SAX: Start of Element !
                             */
                            if !safe_ctxt.sax.is_null()
                                && (*safe_ctxt.sax).startElementNs.is_some()
                                && safe_ctxt.disableSAX == 0
                            {
                                if nbNs > 0 {
                                    (*safe_ctxt.sax)
                                        .startElementNs
                                        .expect("non-null function pointer")(
                                        safe_ctxt.userData,
                                        localname,
                                        prefix,
                                        nsname,
                                        nbNs,
                                        &mut *safe_ctxt
                                            .nsTab
                                            .offset((safe_ctxt.nsNr - 2 * nbNs) as isize),
                                        nbatts / 5,
                                        nbdef,
                                        atts,
                                    );
                                } else {
                                    (*safe_ctxt.sax)
                                        .startElementNs
                                        .expect("non-null function pointer")(
                                        safe_ctxt.userData,
                                        localname,
                                        prefix,
                                        nsname,
                                        0,
                                        0 as *mut *const xmlChar,
                                        nbatts / 5,
                                        nbdef,
                                        atts,
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
        _ => {}
    }
    /*
     * Free up attribute allocated strings if needed
     */
    if attval != 0 {
        i = 3;
        j = 0;
        while j < nratts {
            unsafe {
                if *safe_ctxt.attallocs.offset(j as isize) != 0
                    && !(*atts.offset(i as isize)).is_null()
                {
                    xmlFree_safe(*atts.offset(i as isize) as *mut xmlChar as *mut ());
                }
            }
            i += 5;
            j += 1
        }
    }
    return localname;
}
/* *
* xmlParseEndTag2:
* @ctxt:  an XML parser context
* @line:  line of the start tag
* @nsNr:  number of namespaces on the start tag
*
* parse an end of tag
*
* [42] ETag ::= '</' Name S? '>'
*
* With namespace
*
* [NS 9] ETag ::= '</' QName S? '>'
*/
fn xmlParseEndTag2(mut ctxt: xmlParserCtxtPtr, mut tag: *const xmlStartTag) {
    let mut name: *const xmlChar = 0 as *const xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_tag = unsafe { *tag };
    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }
    if unsafe {
        *(*safe_ctxt.input).cur != '<' as u8 || *(*safe_ctxt.input).cur.offset(1) != '/' as u8
    } {
        unsafe { xmlFatalErr(ctxt, XML_ERR_LTSLASH_REQUIRED, 0 as *const i8) };
        return;
    }
    unsafe {
        (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2);
        (*safe_ctxt.input).col += 2;
        if *(*safe_ctxt.input).cur == 0 {
            xmlParserInputGrow_safe(safe_ctxt.input, 250);
        }
    }

    if safe_tag.prefix.is_null() {
        name = xmlParseNameAndCompare(ctxt, safe_ctxt.name)
    } else {
        name = xmlParseQNameAndCompare(ctxt, safe_ctxt.name, safe_tag.prefix)
    }
    /*
     * We should definitely be at the ending "S? '>'" part
     */
    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return;
    }
    xmlSkipBlankChars(ctxt);
    if unsafe {
        !(0x9 <= *(*safe_ctxt.input).cur as i32 && *(*safe_ctxt.input).cur as i32 <= 0xa
            || *(*safe_ctxt.input).cur as i32 == 0xd
            || 0x20 <= *(*safe_ctxt.input).cur as i32)
            || *(*safe_ctxt.input).cur != '>' as u8
    } {
        unsafe { xmlFatalErr(ctxt, XML_ERR_GT_REQUIRED, 0 as *const i8) };
    } else {
        unsafe {
            (*safe_ctxt.input).col += 1;
            (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(1);
            if *(*safe_ctxt.input).cur == 0 {
                xmlParserInputGrow_safe(safe_ctxt.input, 250);
            }
        }
    }
    /*
     * [ WFC: Element Type Match ]
     * The Name in an element's end-tag must match the element type in the
     * start-tag.
     *
     */
    if name != 1 as *mut xmlChar {
        if name.is_null() {
            name = b"unparsable\x00" as *const u8 as *const i8 as *mut xmlChar
        }
        xmlFatalErrMsgStrIntStr(
            ctxt,
            XML_ERR_TAG_NAME_MISMATCH,
            b"Opening and ending tag mismatch: %s line %d and %s\n\x00" as *const u8 as *const i8,
            safe_ctxt.name,
            safe_tag.line,
            name,
        );
    }
    /*
     * SAX: End of Tag
     */

    if !safe_ctxt.sax.is_null()
        && unsafe { (*safe_ctxt.sax).endElementNs.is_some() }
        && safe_ctxt.disableSAX == 0
    {
        unsafe {
            (*safe_ctxt.sax)
                .endElementNs
                .expect("non-null function pointer")(
                safe_ctxt.userData,
                safe_ctxt.name,
                safe_tag.prefix,
                safe_tag.URI,
            )
        };
    }

    spacePop(ctxt);
    if safe_tag.nsNr != 0 {
        nsPop(ctxt, safe_tag.nsNr);
    }
}
/* *
* xmlParseCDSect:
* @ctxt:  an XML parser context
*
* Parse escaped pure raw content.
*
* [18] CDSect ::= CDStart CData CDEnd
*
* [19] CDStart ::= '<![CDATA['
*
* [20] Data ::= (Char* - (Char* ']]>' Char*))
*
* [21] CDEnd ::= ']]>'
*/

pub fn xmlParseCDSect(mut ctxt: xmlParserCtxtPtr) {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = 100;
    let mut r: i32;
    let mut rl: i32 = 0;
    let mut s: i32;
    let mut sl: i32 = 0;
    let mut cur: i32;
    let mut l: i32 = 0;
    let mut count: i32 = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };

    /* Check 2.6.0 was NXT(0) not RAW */

    if unsafe {
        *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '!' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(2) == '[' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'C' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'D' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(5) == 'A' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(6) == 'T' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(7) == 'A' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(8) == '[' as u8
    } {
        safe_input.cur = unsafe { safe_input.cur.offset(9) };
        safe_input.col += 9;
        if unsafe { *safe_input.cur } == 0 {
            unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
        }
    } else {
        return;
    }

    safe_ctxt.instate = XML_PARSER_CDATA_SECTION;
    r = xmlCurrentChar(ctxt, &mut rl);
    if (if r < 0x100 {
        (0x9 <= r && r <= 0xa || r == 0xd || 0x20 <= r) as i32
    } else {
        (0x100 <= r && r <= 0xd7ff || 0xe000 <= r && r <= 0xfffd || 0x10000 <= r && r <= 0x10ffff)
            as i32
    }) == 0
    {
        unsafe { xmlFatalErr(ctxt, XML_ERR_CDATA_NOT_FINISHED, 0 as *const i8) };
        safe_ctxt.instate = XML_PARSER_CONTENT;
        return;
    }
    if unsafe { *safe_input.cur == '\n' as u8 } {
        safe_input.line += 1;
        safe_input.col = 1
    } else {
        safe_input.col += 1
    }
    safe_input.cur = unsafe { safe_input.cur.offset(rl as isize) };
    s = xmlCurrentChar(ctxt, &mut sl);
    if (if s < 0x100 {
        (0x9 <= s && s <= 0xa || s == 0xd || 0x20 <= s) as i32
    } else {
        (0x100 <= s && s <= 0xd7ff || 0xe000 <= s && s <= 0xfffd || 0x10000 <= s && s <= 0x10ffff)
            as i32
    }) == 0
    {
        unsafe { xmlFatalErr(ctxt, XML_ERR_CDATA_NOT_FINISHED, 0 as *const i8) };
        safe_ctxt.instate = XML_PARSER_CONTENT;
        return;
    }
    if unsafe { *safe_input.cur } == '\n' as u8 {
        safe_input.line += 1;
        safe_input.col = 1
    } else {
        safe_input.col += 1
    }
    safe_input.cur = unsafe { safe_input.cur.offset(sl as isize) };
    cur = xmlCurrentChar(ctxt, &mut l);
    buf =
        unsafe { xmlMallocAtomic_safe(size as u64 * size_of::<xmlChar>() as u64) } as *mut xmlChar;
    if buf.is_null() {
        xmlErrMemory(ctxt, 0 as *const i8);
        return;
    }
    while (if cur < 0x100 {
        (0x9 <= cur && cur <= 0xa || cur == 0xd || 0x20 <= cur) as i32
    } else {
        (0x100 <= cur && cur <= 0xd7ff
            || 0xe000 <= cur && cur <= 0xfffd
            || 0x10000 <= cur && cur <= 0x10ffff) as i32
    }) != 0
        && (r != ']' as i32 || s != ']' as i32 || cur != '>' as i32)
    {
        if len + 5 >= size {
            let mut tmp: *mut xmlChar = 0 as *mut xmlChar;
            if size as u64 > XML_MAX_TEXT_LENGTH && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0 {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_CDATA_NOT_FINISHED,
                    b"CData section too big found\x00" as *const u8 as *const i8,
                    0 as *const xmlChar,
                );
                unsafe { xmlFree_safe(buf as *mut ()) };
                return;
            }
            tmp = unsafe {
                xmlRealloc_safe(
                    buf as *mut (),
                    ((size * 2) as u64 * size_of::<xmlChar>() as u64),
                )
            } as *mut xmlChar;
            if tmp.is_null() {
                unsafe { xmlFree_safe(buf as *mut ()) };
                xmlErrMemory(ctxt, 0 as *const i8);
                return;
            }
            buf = tmp;
            size *= 2
        }
        if rl == 1 {
            unsafe { *buf.offset(len as isize) = r as xmlChar }
            len = len + 1;
        } else {
            len += unsafe { xmlCopyCharMultiByte(&mut *buf.offset(len as isize), r) };
        }
        r = s;
        rl = sl;
        s = cur;
        sl = l;
        count += 1;
        if count > 50 {
            if safe_ctxt.progressive == 0
                && unsafe {
                    (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 > 2 * 250
                        && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64)
                            < 2 * 250
                }
            {
                xmlSHRINK(ctxt);
            }
            if safe_ctxt.progressive == 0
                && unsafe {
                    ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
                }
            {
                xmlGROW(ctxt);
            }
            if safe_ctxt.instate == XML_PARSER_EOF {
                unsafe { xmlFree_safe(buf as *mut ()) };
                return;
            }
            count = 0
        }
        if unsafe { *safe_input.cur } == '\n' as u8 {
            safe_input.line += 1;
            safe_input.col = 1
        } else {
            safe_input.col += 1
        }
        safe_input.cur = unsafe { safe_input.cur.offset(l as isize) };
        cur = xmlCurrentChar(ctxt, &mut l)
    }
    unsafe {
        *buf.offset(len as isize) = 0 as xmlChar;
    }
    safe_ctxt.instate = XML_PARSER_CONTENT;
    if cur != '>' as i32 {
        xmlFatalErrMsgStr(
            ctxt,
            XML_ERR_CDATA_NOT_FINISHED,
            b"CData section not finished\n%.50s\n\x00" as *const u8 as *const i8,
            buf,
        );
        unsafe { xmlFree_safe(buf as *mut ()) };
        return;
    }
    if unsafe { *safe_input.cur } == '\n' as u8 {
        safe_input.line += 1;
        safe_input.col = 1
    } else {
        safe_input.col += 1
    }
    safe_input.cur = unsafe { safe_input.cur.offset(l as isize) };
    /*
     * OK the buffer is to be consumed as cdata.
     */
    if !safe_ctxt.sax.is_null() && safe_ctxt.disableSAX == 0 {
        unsafe {
            if (*safe_ctxt.sax).cdataBlock.is_some() {
                (*safe_ctxt.sax)
                    .cdataBlock
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, buf, len
                );
            } else if (*safe_ctxt.sax).characters.is_some() {
                (*safe_ctxt.sax)
                    .characters
                    .expect("non-null function pointer")(
                    safe_ctxt.userData, buf, len
                );
            }
        }
    }
    unsafe { xmlFree_safe(buf as *mut ()) };
}
/* *
* xmlParseContentInternal:
* @ctxt:  an XML parser context
*
* Parse a content sequence. Stops at EOF or '</'. Leaves checking of
* unexpected EOF to the caller.
*/
fn xmlParseContentInternal(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { &mut *safe_ctxt.input };
    let mut nameNr: i32 = safe_ctxt.nameNr;

    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }
    while unsafe { *safe_input.cur } as i32 != 0 && safe_ctxt.instate != XML_PARSER_EOF {
        let test: *const xmlChar = safe_input.cur;
        let cons: u64 = safe_input.consumed;
        let cur: *const xmlChar = safe_input.cur;

        /*
         * First case : a Processing Instruction.
         */
        if unsafe { *cur == '<' as u8 && *cur.offset(1) == '?' as u8 } {
            xmlParsePI(ctxt);
        }
        /*
         * Second case : a CDSection
         */
        /* 2.6.0 test was *cur not RAW */
        else if unsafe {
            *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '!' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(2) == '[' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'C' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'D' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(5) == 'A' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(6) == 'T' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(7) == 'A' as u8
                && *((*safe_ctxt.input).cur as *mut u8).offset(8) == '[' as u8
        } {
            xmlParseCDSect(ctxt);
        }
        /*
         * Third case :  a comment
         */
        else if unsafe {
            *cur == '<' as u8
                && *(*safe_ctxt.input).cur.offset(1) == '!' as u8
                && *(*safe_ctxt.input).cur.offset(2) == '-' as u8
                && *(*safe_ctxt.input).cur.offset(3) == '-' as u8
        } {
            xmlParseComment(ctxt);
            safe_ctxt.instate = XML_PARSER_CONTENT
        }
        /*
         * Fourth case :  a sub-element.
         */
        else if unsafe { *cur } == '<' as u8 {
            if unsafe { *(*safe_ctxt.input).cur.offset(1) } == '/' as u8 {
                if safe_ctxt.nameNr <= nameNr {
                    break;
                }
                unsafe { xmlParseElementEnd(ctxt) };
            } else {
                unsafe { xmlParseElementStart(ctxt) };
            }
        }
        /*
         * Fifth case : a reference. If if has not been resolved,
         *    parsing returns it's Name, create the node
         */
        else if unsafe { *cur } == '&' as u8 {
            xmlParseReference(ctxt);
        }
        /*
         * Last case, text. Note that References are handled directly.
         */
        else {
            xmlParseCharData(ctxt, 0);
        }
        if safe_ctxt.progressive == 0
            && unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) as i64 } < 250
        {
            xmlGROW(ctxt);
        }
        if safe_ctxt.progressive == 0
            && unsafe { (*safe_ctxt.input).cur.offset_from(safe_input.base) } as i64 > 2 * 250
            && unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) as i64 } < 2 * 250
        {
            xmlSHRINK(ctxt);
        }
        if !(cons == safe_input.consumed && test == safe_input.cur) {
            continue;
        }
        unsafe {
            xmlFatalErr(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"detected an error in element content\n\x00" as *const u8 as *const i8,
            );
            xmlHaltParser(ctxt);
        }
        break;
    }
}
/* *
* xmlParseContent:
* @ctxt:  an XML parser context
*
* Parse a content sequence. Stops at EOF or '</'.
*
* [43] content ::= (element | CharData | Reference | CDSect | PI | Comment)*
*/

pub fn xmlParseContent(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let nameNr: i32 = safe_ctxt.nameNr;
    xmlParseContentInternal(ctxt);
    if safe_ctxt.instate != XML_PARSER_EOF && safe_ctxt.nameNr > nameNr {
        let mut name: *const xmlChar =
            unsafe { *safe_ctxt.nameTab.offset((safe_ctxt.nameNr - 1) as isize) };
        let mut line: i32 =
            unsafe { (*safe_ctxt.pushTab.offset((safe_ctxt.nameNr - 1) as isize)).line };
        xmlFatalErrMsgStrIntStr(
            ctxt,
            XML_ERR_TAG_NOT_FINISHED,
            b"Premature end of data in tag %s line %d\n\x00" as *const u8 as *const i8,
            name,
            line,
            0 as *const xmlChar,
        );
    };
}
/* *
* xmlParseElement:
* @ctxt:  an XML parser context
*
* parse an XML element
*
* [39] element ::= EmptyElemTag | STag content ETag
*
* [ WFC: Element Type Match ]
* The Name in an element's end-tag must match the element type in the
* start-tag.
*
*/

pub fn xmlParseElement(mut ctxt: xmlParserCtxtPtr) {
    if unsafe { xmlParseElementStart(ctxt) } != 0 {
        return;
    }
    xmlParseContentInternal(ctxt);
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if safe_ctxt.instate == XML_PARSER_EOF {
        return;
    }
    if unsafe { *(*safe_ctxt.input).cur } == 0 {
        let mut name: *const xmlChar =
            unsafe { *safe_ctxt.nameTab.offset((safe_ctxt.nameNr - 1) as isize) };
        let mut line: i32 =
            unsafe { (*safe_ctxt.pushTab.offset((safe_ctxt.nameNr - 1) as isize)).line };
        xmlFatalErrMsgStrIntStr(
            ctxt,
            XML_ERR_TAG_NOT_FINISHED,
            b"Premature end of data in tag %s line %d\n\x00" as *const u8 as *const i8,
            name,
            line,
            0 as *const xmlChar,
        );
        return;
    }
    unsafe { xmlParseElementEnd(ctxt) };
}
/* *
* xmlParseElementStart:
* @ctxt:  an XML parser context
*
* Parse the start of an XML element. Returns -1 in case of error, 0 if an
* opening tag was parsed, 1 if an empty element was parsed.
*/
fn xmlParseElementStart(mut ctxt: xmlParserCtxtPtr) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut name: *const xmlChar;
    let mut prefix: *const xmlChar = 0 as *const xmlChar;
    let mut URI: *const xmlChar = 0 as *const xmlChar;
    let mut node_info: xmlParserNodeInfo = xmlParserNodeInfo {
        node: 0 as *const _xmlNode,
        begin_pos: 0,
        begin_line: 0,
        end_pos: 0,
        end_line: 0,
    };
    let mut line: i32 = 0;
    let mut tlen: i32 = 0;
    let ret: xmlNodePtr;
    let nsNr: i32 = safe_ctxt.nsNr;
    let mut safe_input = unsafe { &mut *safe_ctxt.input };

    if safe_ctxt.nameNr as u32 > unsafe { xmlParserMaxDepth }
        && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
    {
        xmlFatalErrMsgInt(
            ctxt,
            XML_ERR_INTERNAL_ERROR,
            b"Excessive depth in document: %d use XML_PARSE_HUGE option\n\x00" as *const u8
                as *const i8,
            unsafe { xmlParserMaxDepth as i32 },
        );
        unsafe {
            xmlHaltParser(ctxt);
        }
        return -1;
    }
    /* Capture start position */
    if safe_ctxt.record_info != 0 {
        node_info.begin_pos = safe_input.consumed
            + unsafe { (*(*ctxt).input).cur.offset_from(safe_input.base) } as u64;
        node_info.begin_line = safe_input.line as u64
    }
    if safe_ctxt.spaceNr == 0 || unsafe { *safe_ctxt.space } == -2 {
        spacePush(ctxt, -1);
    } else {
        unsafe {
            spacePush(ctxt, *safe_ctxt.space);
        }
    }
    line = safe_input.line;

    match () {
        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
        _ => {
            if safe_ctxt.sax2 != 0 {
                /* LIBXML_SAX1_ENABLED */
                name = xmlParseStartTag2(ctxt, &mut prefix, &mut URI, &mut tlen)
            } else {
                name = xmlParseStartTag(ctxt)
            }
        }
        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
        _ => {
            name = xmlParseStartTag2(ctxt, &mut prefix, &mut URI, &mut tlen);
        }
    };

    /* LIBXML_SAX1_ENABLED */
    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }
    if name.is_null() {
        spacePop(ctxt);
        return -1;
    }
    nameNsPush(ctxt, name, prefix, URI, line, safe_ctxt.nsNr - nsNr);
    ret = safe_ctxt.node;

    match () {
        #[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
        _ => {
            /*
             * [ VC: Root Element Type ]
             * The Name in the document type declaration must match the element
             * type of the root element.
             */
            if safe_ctxt.validate != 0
                && safe_ctxt.wellFormed != 0
                && !safe_ctxt.myDoc.is_null()
                && !safe_ctxt.node.is_null()
                && safe_ctxt.node == unsafe { (*safe_ctxt.myDoc).children }
            {
                safe_ctxt.valid &=
                    unsafe { xmlValidateRoot_safe(&mut safe_ctxt.vctxt, safe_ctxt.myDoc) }
            }
        }
        #[cfg(not(HAVE_parser_LIBXML_VALID_ENABLED))]
        _ => {}
    };
    /* LIBXML_VALID_ENABLED */

    /*
     * Check for an Empty Element.
     */
    if unsafe { *safe_input.cur } == '/' as u8 && unsafe { *safe_input.cur.offset(1) } == '>' as u8
    {
        safe_input.cur = unsafe { (*(*ctxt).input).cur.offset(2) };
        safe_input.col += 2;
        if unsafe { *safe_input.cur } as i32 == 0 {
            unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
        }
        if safe_ctxt.sax2 != 0 {
            if !safe_ctxt.sax.is_null()
                && unsafe { (*safe_ctxt.sax).endElementNs.is_some() }
                && safe_ctxt.disableSAX == 0
            {
                unsafe {
                    (*safe_ctxt.sax)
                        .endElementNs
                        .expect("non-null function pointer")(
                        safe_ctxt.userData, name, prefix, URI
                    )
                };
            }
        } else {
            match () {
                #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
                _ => {
                    if !safe_ctxt.sax.is_null()
                        && unsafe { (*safe_ctxt.sax).endElement.is_some() }
                        && safe_ctxt.disableSAX == 0
                    {
                        unsafe {
                            (*safe_ctxt.sax)
                                .endElement
                                .expect("non-null function pointer")(
                                safe_ctxt.userData, name
                            )
                        };
                    }
                }
                #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
                _ => {}
            };
            /* LIBXML_SAX1_ENABLED */
        }
        namePop(ctxt);
        spacePop(ctxt);
        if nsNr != safe_ctxt.nsNr {
            nsPop(ctxt, safe_ctxt.nsNr - nsNr);
        }
        if !ret.is_null() && safe_ctxt.record_info != 0 {
            node_info.end_pos = safe_input.consumed
                + unsafe { (*(*ctxt).input).cur.offset_from(safe_input.base) } as u64;
            node_info.end_line = safe_input.line as u64;
            node_info.node = ret as *const _xmlNode;
            unsafe { xmlParserAddNodeInfo_safe(ctxt, &mut node_info) };
        }
        return 1;
    }
    if unsafe { *safe_input.cur } == '>' as u8 {
        safe_input.col += 1;
        safe_input.cur = unsafe { (*(*ctxt).input).cur.offset(1) };
        if unsafe { *safe_input.cur } as i32 == 0 {
            unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
        }
    } else {
        xmlFatalErrMsgStrIntStr(
            ctxt,
            XML_ERR_GT_REQUIRED,
            b"Couldn\'t find end of Start Tag %s line %d\n\x00" as *const u8 as *const i8,
            name,
            line,
            0 as *const xmlChar,
        );

        /*
         * end of parsing of this node.
         */
        unsafe { nodePop(ctxt) };
        namePop(ctxt);
        spacePop(ctxt);
        if nsNr != safe_ctxt.nsNr {
            nsPop(ctxt, safe_ctxt.nsNr - nsNr);
        }
        /*
         * Capture end position and add node
         */
        if !ret.is_null() && safe_ctxt.record_info != 0 {
            node_info.end_pos = safe_input.consumed
                + unsafe { (*(*ctxt).input).cur.offset_from(safe_input.base) } as u64;
            node_info.end_line = safe_input.line as u64;
            node_info.node = ret as *const _xmlNode;
            unsafe { xmlParserAddNodeInfo_safe(ctxt, &mut node_info) };
        }
        return -1;
    }

    return 0;
}
/* *
* xmlParseElementEnd:
* @ctxt:  an XML parser context
*
* Parse the end of an XML element.
*/
fn xmlParseElementEnd(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };
    let mut node_info: xmlParserNodeInfo = xmlParserNodeInfo {
        node: 0 as *const _xmlNode,
        begin_pos: 0,
        begin_line: 0,
        end_pos: 0,
        end_line: 0,
    };
    let ret: xmlNodePtr = safe_ctxt.node;
    if safe_ctxt.nameNr <= 0 {
        return;
    }
    /*
     * parse the end of tag: '</' should be here.
     */
    if safe_ctxt.sax2 != 0 {
        unsafe {
            xmlParseEndTag2(
                ctxt,
                &mut *(*ctxt).pushTab.offset((safe_ctxt.nameNr - 1) as isize),
            );
        }
        namePop(ctxt);
    } else {
        match () {
            #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
            _ => {
                xmlParseEndTag1(ctxt, 0);
            }
            #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
            _ => {}
        };
    }
    /* LIBXML_SAX1_ENABLED */
    /*
     * Capture end position and add node
     */
    if !ret.is_null() && safe_ctxt.record_info != 0 {
        unsafe {
            node_info.end_pos =
                safe_input.consumed + (*(*ctxt).input).cur.offset_from(safe_input.base) as u64;
            node_info.end_line = safe_input.line as u64;
        }
        node_info.node = ret as *const _xmlNode;
        unsafe { xmlParserAddNodeInfo_safe(ctxt, &mut node_info) };
    };
}
/* *
* xmlParseVersionNum:
* @ctxt:  an XML parser context
*
* parse the XML version value.
*
* [26] VersionNum ::= '1.' [0-9]+
*
* In practice allow [0-9].[0-9]+ at that level
*
* Returns the string giving the XML version number, or NULL
*/

pub fn xmlParseVersionNum(mut ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = 10;
    let mut cur: xmlChar = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };
    buf = unsafe { xmlMallocAtomic_safe((size as u64) * size_of::<xmlChar>() as u64) }
        as *mut xmlChar;
    if buf.is_null() {
        xmlErrMemory(ctxt, 0 as *const i8);
        return 0 as *mut xmlChar;
    }
    cur = unsafe { *safe_input.cur };
    if !(cur >= '0' as u8 && cur <= '9' as u8) {
        unsafe { xmlFree_safe(buf as *mut ()) };
        return 0 as *mut xmlChar;
    }
    unsafe { *buf.offset(len as isize) = cur };
    len = len + 1;
    unsafe { xmlNextChar_safe(ctxt) };
    unsafe { cur = *(*safe_ctxt.input).cur };
    if cur != '.' as u8 {
        unsafe { xmlFree_safe(buf as *mut ()) };
        return 0 as *mut xmlChar;
    }
    unsafe { *buf.offset(len as isize) = cur };
    len = len + 1;

    unsafe { xmlNextChar_safe(ctxt) };
    unsafe { cur = *(*safe_ctxt.input).cur };
    while cur >= '0' as u8 && cur <= '9' as u8 {
        if len + 1 >= size {
            let mut tmp: *mut xmlChar = 0 as *mut xmlChar;
            size *= 2;
            tmp = unsafe {
                xmlRealloc_safe(
                    buf as *mut (),
                    (size as u64).wrapping_mul(size_of::<xmlChar>() as u64),
                )
            } as *mut xmlChar;
            if tmp.is_null() {
                unsafe { xmlFree_safe(buf as *mut ()) };
                xmlErrMemory(ctxt, 0 as *const i8);
                return 0 as *mut xmlChar;
            }
            buf = tmp
        }
        unsafe { *buf.offset(len as isize) = cur };
        len = len + 1;
        unsafe {
            xmlNextChar_safe(ctxt);
            cur = *(*safe_ctxt.input).cur
        }
    }
    unsafe {
        *buf.offset(len as isize) = 0 as xmlChar;
    }
    return buf;
}
/* *
* xmlParseVersionInfo:
* @ctxt:  an XML parser context
*
* parse the XML version.
*
* [24] VersionInfo ::= S 'version' Eq (' VersionNum ' | " VersionNum ")
*
* [25] Eq ::= S? '=' S?
*
* Returns the version string, e.g. "1.0"
*/

pub fn xmlParseVersionInfo(mut ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut version: *mut xmlChar = 0 as *mut xmlChar;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };

    if unsafe {
        *((*(*ctxt).input).cur as *mut u8).offset(0) == 'v' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(1) == 'e' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(2) == 'r' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(3) == 's' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(4) == 'i' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(5) == 'o' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(6) == 'n' as u8
    } {
        unsafe { (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(7) };
        safe_input.col += 7;
        if unsafe { *(*(*ctxt).input).cur } == 0 {
            unsafe { xmlParserInputGrow_safe((*ctxt).input, 250) };
        }
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*(*ctxt).input).cur } != '=' as u8 {
            unsafe { xmlFatalErr(ctxt, XML_ERR_EQUAL_REQUIRED, 0 as *const i8) };
            return 0 as *mut xmlChar;
        }
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*(*ctxt).input).cur } == '\"' as u8 {
            unsafe { xmlNextChar_safe(ctxt) };
            version = xmlParseVersionNum(ctxt);
            if unsafe { *(*(*ctxt).input).cur } != '\"' as u8 {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8) };
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else if unsafe { *(*(*ctxt).input).cur } == '\'' as u8 {
            unsafe { xmlNextChar_safe(ctxt) };
            version = xmlParseVersionNum(ctxt);
            if unsafe { *(*(*ctxt).input).cur } != '\'' as u8 {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8) };
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_STRING_NOT_STARTED, 0 as *const i8);
            }
        }
    }
    return version;
}
/* *
* xmlParseEncName:
* @ctxt:  an XML parser context
*
* parse the XML encoding name
*
* [81] EncName ::= [A-Za-z] ([A-Za-z0-9._] | '-')*
*
* Returns the encoding name value or NULL
*/

pub fn xmlParseEncName(mut ctxt: xmlParserCtxtPtr) -> *mut xmlChar {
    let mut buf: *mut xmlChar = 0 as *mut xmlChar;
    let mut len: i32 = 0;
    let mut size: i32 = 10;
    let mut cur: xmlChar = 0;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };

    cur = unsafe { *(*safe_ctxt.input).cur };
    if cur >= 'a' as u8 && cur <= 'z' as u8 || cur >= 'A' as u8 && cur <= 'Z' as u8 {
        buf = unsafe { xmlMallocAtomic_safe((size as u64) * size_of::<xmlChar>() as u64) }
            as *mut xmlChar;
        if buf.is_null() {
            xmlErrMemory(ctxt, 0 as *const i8);
            return 0 as *mut xmlChar;
        }
        unsafe {
            *buf.offset(len as isize) = cur;
        }
        len = len + 1;
        unsafe { xmlNextChar_safe(ctxt) };
        cur = unsafe { *(*safe_ctxt.input).cur };
        while cur >= 'a' as u8 && cur <= 'z' as u8
            || cur >= 'A' as u8 && cur <= 'Z' as u8
            || cur >= '0' as u8 && cur <= '9' as u8
            || cur == '.' as u8
            || cur == '_' as u8
            || cur == '-' as u8
        {
            if len + 1 >= size {
                let tmp: *mut xmlChar;
                size *= 2;
                tmp = unsafe {
                    xmlRealloc_safe(buf as *mut (), (size as u64) * size_of::<xmlChar>() as u64)
                } as *mut xmlChar;
                if tmp.is_null() {
                    xmlErrMemory(ctxt, 0 as *const i8);
                    unsafe { xmlFree_safe(buf as *mut ()) };
                    return 0 as *mut xmlChar;
                }
                buf = tmp
            }
            unsafe {
                *buf.offset(len as isize) = cur;
            }
            len = len + 1;
            unsafe { xmlNextChar_safe(ctxt) };
            cur = unsafe { *(*safe_ctxt.input).cur };
            if cur == 0 {
                if safe_ctxt.progressive == 0
                    && unsafe {
                        (*safe_ctxt.input).cur.offset_from(safe_input.base) as i64 > 2 * 250
                            && ((*safe_ctxt.input).end.offset_from(safe_input.cur) as i64) < 2 * 250
                    }
                {
                    xmlSHRINK(ctxt);
                }
                if safe_ctxt.progressive == 0
                    && unsafe { (*safe_ctxt.input).end.offset_from(safe_input.cur) as i64 } < 250
                {
                    xmlGROW(ctxt);
                }
                unsafe { cur = *(*safe_ctxt.input).cur }
            }
        }
        unsafe { *buf.offset(len as isize) = 0 as xmlChar }
    } else {
        unsafe { xmlFatalErr(ctxt, XML_ERR_ENCODING_NAME, 0 as *const i8) };
    }
    return buf;
}
/* *
* xmlParseEncodingDecl:
* @ctxt:  an XML parser context
*
* parse the XML encoding declaration
*
* [80] EncodingDecl ::= S 'encoding' Eq ('"' EncName '"' |  "'" EncName "'")
*
* this setups the conversion filters.
*
* Returns the encoding value or NULL
*/

pub fn xmlParseEncodingDecl(mut ctxt: xmlParserCtxtPtr) -> *const xmlChar {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };
    let mut encoding: *mut xmlChar = 0 as *mut xmlChar;
    xmlSkipBlankChars(ctxt);
    if unsafe {
        *((*safe_ctxt.input).cur as *mut u8).offset(0) == 'e' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(1) == 'n' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(2) == 'c' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'o' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'd' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(5) == 'i' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(6) == 'n' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(7) == 'g' as u8
    } {
        unsafe {
            (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(8);
            safe_input.col += 8;
            if *(*safe_ctxt.input).cur == 0 {
                xmlParserInputGrow_safe(safe_ctxt.input, 250);
            }
        }
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*safe_ctxt.input).cur != '=' as u8 } {
            unsafe { xmlFatalErr(ctxt, XML_ERR_EQUAL_REQUIRED, 0 as *const i8) };
            return 0 as *const xmlChar;
        }
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*safe_ctxt.input).cur == '\"' as u8 } {
            unsafe { xmlNextChar_safe(ctxt) };
            encoding = xmlParseEncName(ctxt);
            if unsafe { *(*safe_ctxt.input).cur != '\"' as u8 } {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8);
                    xmlFree_safe(encoding as *mut ())
                };
                return 0 as *const xmlChar;
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else if unsafe { *(*safe_ctxt.input).cur == '\'' as u8 } {
            unsafe { xmlNextChar_safe(ctxt) };
            encoding = xmlParseEncName(ctxt);
            if unsafe { *(*safe_ctxt.input).cur != '\'' as u8 } {
                unsafe {
                    xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8);
                    xmlFree_safe(encoding as *mut ())
                };
                return 0 as *const xmlChar;
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else {
            unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_STARTED, 0 as *const i8) };
        }
        /*
         * Non standard parsing, allowing the user to ignore encoding
         */
        if safe_ctxt.options & XML_PARSE_IGNORE_ENC as i32 != 0 {
            unsafe { xmlFree_safe(encoding as *mut ()) };
            return 0 as *const xmlChar;
        }
        /*
         * UTF-16 encoding switch has already taken place at this stage,
         * more over the little-endian/big-endian selection is already done
         */
        if !encoding.is_null()
            && unsafe {
                xmlStrcasecmp_safe(
                    encoding,
                    b"UTF-16\x00" as *const u8 as *const i8 as *mut xmlChar,
                ) == 0
                    || xmlStrcasecmp_safe(
                        encoding,
                        b"UTF16\x00" as *const u8 as *const i8 as *mut xmlChar,
                    ) == 0
            }
        {
            /*
             * If no encoding was passed to the parser, that we are
             * using UTF-16 and no decoder is present i.e. the
             * document is apparently UTF-8 compatible, then raise an
             * encoding mismatch fatal error
             */
            if unsafe {
                safe_ctxt.encoding.is_null()
                    && !(*safe_ctxt.input).buf.is_null()
                    && (*(*safe_ctxt.input).buf).encoder.is_null()
            } {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_INVALID_ENCODING,
                    b"Document labelled UTF-16 but has UTF-8 content\n\x00" as *const u8
                        as *const i8,
                );
            }
            if !safe_ctxt.encoding.is_null() {
                unsafe { xmlFree_safe(safe_ctxt.encoding as *mut xmlChar as *mut ()) };
            }
            safe_ctxt.encoding = encoding
        }
        /*
         * UTF-8 encoding is handled natively
         */
        else if !encoding.is_null()
            && unsafe {
                xmlStrcasecmp_safe(
                    encoding,
                    b"UTF-8\x00" as *const u8 as *const i8 as *mut xmlChar,
                ) == 0
                    || xmlStrcasecmp_safe(
                        encoding,
                        b"UTF8\x00" as *const u8 as *const i8 as *mut xmlChar,
                    ) == 0
            }
        {
            if !safe_ctxt.encoding.is_null() {
                unsafe { xmlFree_safe(safe_ctxt.encoding as *mut xmlChar as *mut ()) };
            }
            safe_ctxt.encoding = encoding
        } else if !encoding.is_null() {
            let mut handler: xmlCharEncodingHandlerPtr = 0 as *mut xmlCharEncodingHandler;
            unsafe {
                if !(*safe_ctxt.input).encoding.is_null() {
                    xmlFree_safe((*safe_ctxt.input).encoding as *mut xmlChar as *mut ());
                }
                (*safe_ctxt.input).encoding = encoding;
            }

            handler = unsafe { xmlFindCharEncodingHandler_safe(encoding as *const i8) };
            if !handler.is_null() {
                if unsafe { xmlSwitchToEncoding_safe(ctxt, handler) } < 0 {
                    /* failed to convert */
                    safe_ctxt.errNo = XML_ERR_UNSUPPORTED_ENCODING as i32;
                    return 0 as *const xmlChar;
                }
            } else {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_UNSUPPORTED_ENCODING,
                    b"Unsupported encoding %s\n\x00" as *const u8 as *const i8,
                    encoding,
                );
                return 0 as *const xmlChar;
            }
        }
    }
    return encoding;
}
/* *
* xmlParseSDDecl:
* @ctxt:  an XML parser context
*
* parse the XML standalone declaration
*
* [32] SDDecl ::= S 'standalone' Eq
*                 (("'" ('yes' | 'no') "'") | ('"' ('yes' | 'no')'"'))
*
* [ VC: Standalone Document Declaration ]
* TODO The standalone document declaration must have the value "no"
* if any external markup declarations contain declarations of:
*  - attributes with default values, if elements to which these
*    attributes apply appear in the document without specifications
*    of values for these attributes, or
*  - entities (other than amp, lt, gt, apos, quot), if references
*    to those entities appear in the document, or
*  - attributes with values subject to normalization, where the
*    attribute appears in the document with a value which will change
*    as a result of normalization, or
*  - element types with element content, if white space occurs directly
*    within any instance of those types.
*
* Returns:
*   1 if standalone="yes"
*   0 if standalone="no"
*  -2 if standalone attribute is missing or invalid
*	  (A standalone value of -2 means that the XML declaration was found,
*	   but no value was specified for the standalone attribute).
*/

pub fn xmlParseSDDecl(mut ctxt: xmlParserCtxtPtr) -> i32 {
    let mut standalone: i32 = -2;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };

    xmlSkipBlankChars(ctxt);

    if unsafe {
        *((*safe_ctxt.input).cur as *mut u8).offset(0) == 's' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(1) == 't' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(2) == 'a' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'n' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'd' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(5) == 'a' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(6) == 'l' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(7) == 'o' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(8) == 'n' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(9) == 'e' as u8
    } {
        unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(10) };
        safe_input.col += 10;
        if unsafe { *(*safe_ctxt.input).cur } == 0 {
            unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
        }
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*safe_ctxt.input).cur } != '=' as u8 {
            unsafe { xmlFatalErr(ctxt, XML_ERR_EQUAL_REQUIRED, 0 as *const i8) };
            return standalone;
        }
        unsafe { xmlNextChar_safe(ctxt) };
        xmlSkipBlankChars(ctxt);
        if unsafe { *(*safe_ctxt.input).cur } == '\'' as u8 {
            unsafe { xmlNextChar_safe(ctxt) };
            if unsafe {
                *(*safe_ctxt.input).cur == 'n' as u8
                    && *(*safe_ctxt.input).cur.offset(1) == 'o' as u8
            } {
                standalone = 0;
                unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2) };
                safe_input.col += 2;
                if unsafe { *(*safe_ctxt.input).cur } == 0 {
                    unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
                }
            } else if unsafe {
                *(*safe_ctxt.input).cur == 'y' as u8
                    && *(*safe_ctxt.input).cur.offset(1) == 'e' as u8
                    && *(*safe_ctxt.input).cur.offset(2) == 's' as u8
            } {
                standalone = 1;
                unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(3) };
                safe_input.col += 3;
                if unsafe { *(*safe_ctxt.input).cur } == 0 {
                    unsafe { xmlParserInputGrow_safe(safe_ctxt.input, 250) };
                }
            } else {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STANDALONE_VALUE, 0 as *const i8) };
            }
            if unsafe { *(*safe_ctxt.input).cur } != '\'' as u8 {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8) };
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else if unsafe { *(*safe_ctxt.input).cur } == '\"' as u8 {
            unsafe { xmlNextChar_safe(ctxt) };
            if unsafe {
                *(*safe_ctxt.input).cur == 'n' as u8
                    && *(*safe_ctxt.input).cur.offset(1) == 'o' as u8
            } {
                standalone = 0;
                unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2) };
                safe_input.col += 2;
                unsafe {
                    if *(*safe_ctxt.input).cur == 0 {
                        xmlParserInputGrow_safe(safe_ctxt.input, 250);
                    }
                }
            } else if unsafe {
                *(*safe_ctxt.input).cur == 'y' as u8
                    && *(*safe_ctxt.input).cur.offset(1) == 'e' as u8
                    && *(*safe_ctxt.input).cur.offset(2) == 's' as u8
            } {
                standalone = 1;
                unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(3) };
                safe_input.col += 3;
                unsafe {
                    if *(*safe_ctxt.input).cur == 0 {
                        xmlParserInputGrow_safe(safe_ctxt.input, 250);
                    }
                }
            } else {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STANDALONE_VALUE, 0 as *const i8) };
            }
            if unsafe { *(*safe_ctxt.input).cur } != '\"' as u8 {
                unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_CLOSED, 0 as *const i8) };
            } else {
                unsafe { xmlNextChar_safe(ctxt) };
            }
        } else {
            unsafe { xmlFatalErr(ctxt, XML_ERR_STRING_NOT_STARTED, 0 as *const i8) };
        }
    }
    return standalone;
}
/* *
* xmlParseXMLDecl:
* @ctxt:  an XML parser context
*
* parse an XML declaration header
*
* [23] XMLDecl ::= '<?xml' VersionInfo EncodingDecl? SDDecl? S? '?>'
*/

pub fn xmlParseXMLDecl(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };
    let mut version: *mut xmlChar = 0 as *mut xmlChar;

    /*
     * This value for standalone indicates that the document has an
     * XML declaration but it does not have a standalone attribute.
     * It will be overwritten later if a standalone attribute is found.
     */
    safe_input.standalone = -2;
    /*
     * We know that '<?xml' is here.
     */
    unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(5) };
    safe_input.col += 5;
    unsafe {
        if *(*safe_ctxt.input).cur == 0 {
            xmlParserInputGrow_safe(safe_ctxt.input, 250);
        }
    }
    if !unsafe {
        *(*safe_ctxt.input).cur == 0x20
            || 0x9 <= *(*safe_ctxt.input).cur && *(*safe_ctxt.input).cur <= 0xa
            || *(*safe_ctxt.input).cur == 0xd
    } {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_SPACE_REQUIRED,
            b"Blank needed after \'<?xml\'\n\x00" as *const u8 as *const i8,
        );
    }

    xmlSkipBlankChars(ctxt);
    /*
     * We must have the VersionInfo here.
     */
    version = xmlParseVersionInfo(ctxt);
    if version.is_null() {
        unsafe { xmlFatalErr(ctxt, XML_ERR_VERSION_MISSING, 0 as *const i8) };
    } else {
        if unsafe {
            xmlStrEqual_safe(
                version,
                b"1.0\x00" as *const u8 as *const i8 as *const xmlChar,
            )
        } == 0
        {
            /*
             * Changed here for XML-1.0 5th edition
             */
            if safe_ctxt.options & XML_PARSE_OLD10 as i32 != 0 {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_UNKNOWN_VERSION,
                    b"Unsupported version \'%s\'\n\x00" as *const u8 as *const i8,
                    version,
                );
            } else if unsafe { *version.offset(0) == '1' as u8 && *version.offset(1) == '.' as u8 }
            {
                xmlWarningMsg(
                    ctxt,
                    XML_WAR_UNKNOWN_VERSION,
                    b"Unsupported version \'%s\'\n\x00" as *const u8 as *const i8,
                    version,
                    0 as *const xmlChar,
                );
            } else {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_UNKNOWN_VERSION,
                    b"Unsupported version \'%s\'\n\x00" as *const u8 as *const i8,
                    version,
                );
            }
        }
        if !safe_ctxt.version.is_null() {
            unsafe { xmlFree_safe(safe_ctxt.version as *mut ()) };
        }
        safe_ctxt.version = version
    }

    /*
     * We may have the encoding declaration
     */
    if !unsafe {
        *(*safe_ctxt.input).cur == 0x20
            || 0x9 <= *(*safe_ctxt.input).cur && *(*safe_ctxt.input).cur <= 0xa
            || *(*safe_ctxt.input).cur == 0xd
    } {
        if unsafe {
            *(*safe_ctxt.input).cur == '?' as u8 && *(*safe_ctxt.input).cur.offset(1) == '>' as u8
        } {
            unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2) };
            safe_input.col += 2;
            unsafe {
                if *(*safe_ctxt.input).cur == 0 {
                    xmlParserInputGrow_safe(safe_ctxt.input, 250);
                }
            }
            return;
        }
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_SPACE_REQUIRED,
            b"Blank needed here\n\x00" as *const u8 as *const i8,
        );
    }

    xmlParseEncodingDecl(ctxt);
    if safe_ctxt.errNo == XML_ERR_UNSUPPORTED_ENCODING as i32 || safe_ctxt.instate == XML_PARSER_EOF
    {
        /*
         * The XML REC instructs us to stop parsing right here
         */
        return;
    }

    /*
     * We may have the standalone status.
     */
    if unsafe {
        !(*safe_ctxt.input).encoding.is_null() && !*(*safe_ctxt.input).cur == 0x20
            || 0x9 <= *(*safe_ctxt.input).cur && *(*safe_ctxt.input).cur <= 0xa
            || *(*safe_ctxt.input).cur == 0xd
    } {
        if unsafe {
            *(*safe_ctxt.input).cur == '?' as u8 && *(*safe_ctxt.input).cur.offset(1) == '>' as u8
        } {
            unsafe { (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2) };
            safe_input.col += 2;
            unsafe {
                if *(*safe_ctxt.input).cur == 0 {
                    xmlParserInputGrow_safe(safe_ctxt.input, 250);
                }
            }
            return;
        }
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_SPACE_REQUIRED,
            b"Blank needed here\n\x00" as *const u8 as *const i8,
        );
    }

    /*
     * We can grow the input buffer freely at that point
     */
    if unsafe {
        safe_ctxt.progressive == 0
            && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    } {
        xmlGROW(ctxt);
    }
    xmlSkipBlankChars(ctxt);
    unsafe { (*safe_ctxt.input).standalone = xmlParseSDDecl(ctxt) };
    xmlSkipBlankChars(ctxt);
    if unsafe {
        *(*safe_ctxt.input).cur == '?' as u8 && *(*safe_ctxt.input).cur.offset(1) == '>' as u8
    } {
        unsafe {
            (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(2);
            safe_input.col += 2;
            if *(*safe_ctxt.input).cur == 0 {
                xmlParserInputGrow_safe(safe_ctxt.input, 250);
            }
        }
    } else if unsafe { *(*safe_ctxt.input).cur == '>' as u8 } {
        /* Deprecated old WD ... */
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_XMLDECL_NOT_FINISHED, 0 as *const i8);
            xmlNextChar_safe(ctxt)
        };
    } else {
        unsafe {
            xmlFatalErr(ctxt, XML_ERR_XMLDECL_NOT_FINISHED, 0 as *const i8);
            while *(*safe_ctxt.input).cur != 0 && *(*safe_ctxt.input).cur != '>' as u8 {
                (*safe_ctxt.input).cur = (*safe_ctxt.input).cur.offset(1)
            }
            xmlNextChar_safe(ctxt)
        }
    };
}
/* *
* xmlParseMisc:
* @ctxt:  an XML parser context
*
* parse an XML Misc* optional field.
*
* [27] Misc ::= Comment | PI |  S
*/

pub fn xmlParseMisc(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    while safe_ctxt.instate != XML_PARSER_EOF
        && unsafe {
            *(*safe_ctxt.input).cur == '<' as u8 && *(*safe_ctxt.input).cur.offset(1) == '?' as u8
                || *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '!' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(2) == '-' as u8
                    && *((*safe_ctxt.input).cur as *mut u8).offset(3) == '-' as u8
                || (*(*safe_ctxt.input).cur == 0x20
                    || 0x9 <= *(*safe_ctxt.input).cur && *(*safe_ctxt.input).cur <= 0xa
                    || *(*safe_ctxt.input).cur == 0xd)
        }
    {
        if unsafe {
            *(*safe_ctxt.input).cur == '<' as u8 && *(*safe_ctxt.input).cur.offset(1) == '?' as u8
        } {
            xmlParsePI(ctxt);
        } else if unsafe {
            *(*safe_ctxt.input).cur == 0x20
                || 0x9 <= *(*safe_ctxt.input).cur && *(*safe_ctxt.input).cur <= 0xa
                || *(*safe_ctxt.input).cur == 0xd
        } {
            unsafe { xmlNextChar_safe(ctxt) };
        } else {
            xmlParseComment(ctxt);
        }
    }
}
/* *
* xmlParseDocument:
* @ctxt:  an XML parser context
*
* parse an XML document (and build a tree if using the standard SAX
* interface).
*
* [1] document ::= prolog element Misc*
*
* [22] prolog ::= XMLDecl? Misc* (doctypedecl Misc*)?
*
* Returns 0, -1 in case of error. the parser context is augmented
*                as a result of the parsing.
*/

pub unsafe fn xmlParseDocument(mut ctxt: xmlParserCtxtPtr) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_input = unsafe { *safe_ctxt.input };
    let mut start: [xmlChar; 4] = [0; 4];
    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
    unsafe {
        xmlInitParser();
    }
    if ctxt.is_null() || safe_ctxt.input.is_null() {
        return -1;
    }
    if safe_ctxt.progressive == 0
        && unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 } < 250
    {
        xmlGROW(ctxt);
    }
    /*
     * SAX: detecting the level.
     */
    xmlDetectSAX2(ctxt);

    /*
     * SAX: beginning of the document processing.
     */
    if !safe_ctxt.sax.is_null() && unsafe { (*safe_ctxt.sax).setDocumentLocator.is_some() } {
        unsafe {
            (*safe_ctxt.sax)
                .setDocumentLocator
                .expect("non-null function pointer")(
                safe_ctxt.userData, __xmlDefaultSAXLocator()
            )
        };
    }

    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }

    if safe_ctxt.encoding.is_null()
        && unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) } as i64 >= 4
    {
        /*
         * Get the 4 first bytes and decode the charset
         * if enc != XML_CHAR_ENCODING_NONE
         * plug some encoding conversion routines.
         */
        unsafe {
            start[0] = *(*safe_ctxt.input).cur;
            start[1] = *(*safe_ctxt.input).cur.offset(1);
            start[2] = *(*safe_ctxt.input).cur.offset(2);
            start[3] = *(*safe_ctxt.input).cur.offset(3);
            enc = xmlDetectCharEncoding_safe(&mut *start.as_mut_ptr().offset(0), 4);
        }
        if enc != XML_CHAR_ENCODING_NONE {
            unsafe { xmlSwitchEncoding_safe(ctxt, enc) };
        }
    }
    if unsafe { *(*safe_ctxt.input).cur } == 0 {
        unsafe { xmlFatalErr(ctxt, XML_ERR_DOCUMENT_EMPTY, 0 as *const i8) };
        return -1;
    }
    /*
     * Check for the XMLDecl in the Prolog.
     * do not GROW here to avoid the detected encoder to decode more
     * than just the first line, unless the amount of data is really
     * too small to hold "<?xml version="1.0" encoding="foo"
     */
    if (unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) } as i64) < 35 {
        if safe_ctxt.progressive == 0
            && (unsafe { (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) } as i64) < 250
        {
            xmlGROW(ctxt);
        }
    }
    if unsafe {
        *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '?' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(2) == 'x' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'm' as u8
            && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'l' as u8
            && (*(*safe_ctxt.input).cur.offset(5) == 0x20
                || 0x9 <= *(*safe_ctxt.input).cur.offset(5)
                    && *(*safe_ctxt.input).cur.offset(5) <= 0xa
                || *(*safe_ctxt.input).cur.offset(5) == 0xd)
    } {
        /*
         * Note that we will switch encoding on the fly.
         */
        xmlParseXMLDecl(ctxt);
        if safe_ctxt.errNo == XML_ERR_UNSUPPORTED_ENCODING as i32
            || safe_ctxt.instate == XML_PARSER_EOF
        {
            /*
             * The XML REC instructs us to stop parsing right here
             */
            return -1;
        }
        safe_ctxt.standalone = (*safe_ctxt.input).standalone;
        xmlSkipBlankChars(ctxt);
    } else {
        safe_ctxt.version = xmlCharStrdup_safe(b"1.0\x00" as *const u8 as *const i8)
    }
    if !safe_ctxt.sax.is_null()
        && (*safe_ctxt.sax).startDocument.is_some()
        && safe_ctxt.disableSAX == 0
    {
        (*safe_ctxt.sax)
            .startDocument
            .expect("non-null function pointer")(safe_ctxt.userData);
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }
    if !safe_ctxt.myDoc.is_null()
        && !safe_ctxt.input.is_null()
        && !(*safe_ctxt.input).buf.is_null()
        && (*(*safe_ctxt.input).buf).compressed >= 0
    {
        (*safe_ctxt.myDoc).compression = (*(*safe_ctxt.input).buf).compressed
    }
    /*
     * The Misc part of the Prolog
     */
    if safe_ctxt.progressive == 0
        && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    {
        xmlGROW(ctxt);
    }
    xmlParseMisc(ctxt);
    /*
     * Then possibly doc type declaration(s) and more Misc
     * (doctypedecl Misc*)?
     */
    if safe_ctxt.progressive == 0
        && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    {
        xmlGROW(ctxt);
    }
    if *((*safe_ctxt.input).cur as *mut u8).offset(0) == '<' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(1) == '!' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(2) == 'D' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(3) == 'O' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(4) == 'C' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(5) == 'T' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(6) == 'Y' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(7) == 'P' as u8
        && *((*safe_ctxt.input).cur as *mut u8).offset(8) == 'E' as u8
    {
        safe_ctxt.inSubset = 1;
        xmlParseDocTypeDecl(ctxt);
        if *(*safe_ctxt.input).cur == '[' as u8 {
            safe_ctxt.instate = XML_PARSER_DTD;
            xmlParseInternalSubset(ctxt);
            if safe_ctxt.instate == XML_PARSER_EOF {
                return -1;
            }
        }
        /*
         * Create and update the external subset.
         */
        safe_ctxt.inSubset = 2 as i32;
        if !safe_ctxt.sax.is_null()
            && (*safe_ctxt.sax).externalSubset.is_some()
            && safe_ctxt.disableSAX == 0
        {
            (*safe_ctxt.sax)
                .externalSubset
                .expect("non-null function pointer")(
                safe_ctxt.userData,
                safe_ctxt.intSubName,
                safe_ctxt.extSubSystem,
                safe_ctxt.extSubURI,
            );
        }
        if safe_ctxt.instate == XML_PARSER_EOF {
            return -1;
        }
        safe_ctxt.inSubset = 0;
        xmlCleanSpecialAttr(ctxt);
        safe_ctxt.instate = XML_PARSER_PROLOG;
        xmlParseMisc(ctxt);
    }
    /*
     * Time to start parsing the tree itself
     */
    if safe_ctxt.progressive == 0
        && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
    {
        xmlGROW(ctxt);
    }
    if *(*safe_ctxt.input).cur != '<' as u8 {
        xmlFatalErrMsg(
            ctxt,
            XML_ERR_DOCUMENT_EMPTY,
            b"Start tag expected, \'<\' not found\n\x00" as *const u8 as *const i8,
        );
    } else {
        safe_ctxt.instate = XML_PARSER_CONTENT;
        xmlParseElement(ctxt);
        safe_ctxt.instate = XML_PARSER_EPILOG;
        /*
         * The Misc part at the end
         */
        xmlParseMisc(ctxt);
        if *(*safe_ctxt.input).cur as i32 != 0 {
            xmlFatalErr(ctxt, XML_ERR_DOCUMENT_END, 0 as *const i8);
        }
        safe_ctxt.instate = XML_PARSER_EOF
    }
    /*
     * SAX: end of the document processing.
     */
    if !safe_ctxt.sax.is_null() && (*safe_ctxt.sax).endDocument.is_some() {
        (*safe_ctxt.sax)
            .endDocument
            .expect("non-null function pointer")(safe_ctxt.userData);
    }
    /*
     * Remove locally kept entity definitions if the tree was not built
     */
    if !safe_ctxt.myDoc.is_null()
        && xmlStrEqual_safe(
            (*safe_ctxt.myDoc).version,
            b"SAX compatibility mode document\x00" as *const u8 as *const i8 as *mut xmlChar,
        ) != 0
    {
        xmlFreeDoc(safe_ctxt.myDoc);
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if safe_ctxt.wellFormed != 0 && !safe_ctxt.myDoc.is_null() {
        (*safe_ctxt.myDoc).properties |= XML_DOC_WELLFORMED as i32;
        if safe_ctxt.valid != 0 {
            (*safe_ctxt.myDoc).properties |= XML_DOC_DTDVALID as i32
        }
        if safe_ctxt.nsWellFormed != 0 {
            (*safe_ctxt.myDoc).properties |= XML_DOC_NSVALID as i32
        }
        if safe_ctxt.options & XML_PARSE_OLD10 as i32 != 0 {
            (*safe_ctxt.myDoc).properties |= XML_DOC_OLD10 as i32
        }
    }

    if safe_ctxt.wellFormed == 0 {
        safe_ctxt.valid = 0;
        return -1;
    }
    return 0;
}
/* *
* xmlParseExtParsedEnt:
* @ctxt:  an XML parser context
*
* parse a general parsed entity
* An external general parsed entity is well-formed if it matches the
* production labeled extParsedEnt.
*
* [78] extParsedEnt ::= TextDecl? content
*
* Returns 0, -1 in case of error. the parser context is augmented
*                as a result of the parsing.
*/

pub unsafe fn xmlParseExtParsedEnt(mut ctxt: xmlParserCtxtPtr) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut start: [xmlChar; 4] = [0; 4];
    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
    if ctxt.is_null() || safe_ctxt.input.is_null() {
        return -1;
    }
    xmlDefaultSAXHandlerInit_safe();
    xmlDetectSAX2(ctxt);
    unsafe {
        if (*ctxt).progressive == 0
            && ((*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64) < 250
        {
            xmlGROW(ctxt);
        }
        /*
         * SAX: beginning of the document processing.
         */
        if !(*ctxt).sax.is_null() && (*(*ctxt).sax).setDocumentLocator.is_some() {
            (*(*ctxt).sax)
                .setDocumentLocator
                .expect("non-null function pointer")(
                (*ctxt).userData, __xmlDefaultSAXLocator()
            );
        }
        /*
         * Get the 4 first bytes and decode the charset
         * if enc != XML_CHAR_ENCODING_NONE
         * plug some encoding conversion routines.
         */
        if (*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64 >= 4 {
            start[0] = *(*(*ctxt).input).cur;
            start[1] = *(*(*ctxt).input).cur.offset(1);
            start[2] = *(*(*ctxt).input).cur.offset(2);
            start[3] = *(*(*ctxt).input).cur.offset(3);
            enc = xmlDetectCharEncoding_safe(start.as_mut_ptr(), 4);
            if enc != XML_CHAR_ENCODING_NONE {
                xmlSwitchEncoding_safe(ctxt, enc);
            }
        }
        if *(*(*ctxt).input).cur as i32 == 0 {
            xmlFatalErr(ctxt, XML_ERR_DOCUMENT_EMPTY, 0 as *const i8);
        }
        /*
         * Check for the XMLDecl in the Prolog.
         */
        if (*ctxt).progressive == 0
            && ((*(*ctxt).input).end.offset_from((*(*ctxt).input).cur) as i64) < 250
        {
            xmlGROW(ctxt);
        }
        if *((*(*ctxt).input).cur as *mut u8).offset(0) == '<' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(1) == '?' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(2) == 'x' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(3) == 'm' as u8
            && *((*(*ctxt).input).cur as *mut u8).offset(4) == 'l' as u8
            && (*(*(*ctxt).input).cur.offset(5) as i32 == 0x20
                || 0x9 <= *(*(*ctxt).input).cur.offset(5) as i32
                    && *(*(*ctxt).input).cur.offset(5) as i32 <= 0xa
                || *(*(*ctxt).input).cur.offset(5) as i32 == 0xd)
        {
            /*
             * Note that we will switch encoding on the fly.
             */
            xmlParseXMLDecl(ctxt);
            if (*ctxt).errNo == XML_ERR_UNSUPPORTED_ENCODING as i32 {
                /*
                 * The XML REC instructs us to stop parsing right here
                 */
                return -1;
            }
            xmlSkipBlankChars(ctxt);
        } else {
            (*ctxt).version = xmlCharStrdup_safe(b"1.0\x00" as *const u8 as *const i8)
        }
        if !(*ctxt).sax.is_null()
            && (*(*ctxt).sax).startDocument.is_some()
            && (*ctxt).disableSAX == 0
        {
            (*(*ctxt).sax)
                .startDocument
                .expect("non-null function pointer")((*ctxt).userData);
        }
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }
    /*
     * Doing validity checking on chunk doesn't make sense
     */
    safe_ctxt.instate = XML_PARSER_CONTENT;
    safe_ctxt.validate = 0;
    safe_ctxt.loadsubset = 0;
    safe_ctxt.depth = 0;
    xmlParseContent(ctxt);
    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }
    unsafe {
        if *(*(*ctxt).input).cur == '<' as u8 && *(*(*ctxt).input).cur.offset(1) == '/' as u8 {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        } else if *(*(*ctxt).input).cur as i32 != 0 {
            xmlFatalErr(ctxt, XML_ERR_EXTRA_CONTENT, 0 as *const i8);
        }
        /*
         * SAX: end of the document processing.
         */
        if !safe_ctxt.sax.is_null() && (*(*ctxt).sax).endDocument.is_some() {
            (*(*ctxt).sax)
                .endDocument
                .expect("non-null function pointer")((*ctxt).userData);
        }
    }
    if safe_ctxt.wellFormed == 0 {
        return -1;
    }
    return 0;
}
/* ***********************************************************************
*									*
*		Progressive parsing interfaces				*
*									*
************************************************************************/
/* *
* xmlParseLookupSequence:
* @ctxt:  an XML parser context
* @first:  the first char to lookup
* @next:  the next char to lookup or zero
* @third:  the next char to lookup or zero
*
* Try to find if a sequence (first, next, third) or  just (first next) or
* (first) is available in the input stream.
* This function has a side effect of (possibly) incrementing ctxt->checkIndex
* to avoid rescanning sequences of bytes, it DOES change the state of the
* parser, do not use liberally.
*
* Returns the index to the current parsing point if the full sequence
*      is available, -1 otherwise.
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
unsafe fn xmlParseLookupSequence(
    mut ctxt: xmlParserCtxtPtr,
    mut first: xmlChar,
    mut next: xmlChar,
    mut third: xmlChar,
) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut base: i32 = 0;
    let mut len: i32 = 0;
    let mut in_0: xmlParserInputPtr = 0 as *mut xmlParserInput;
    let mut buf: *const xmlChar = 0 as *const xmlChar;
    in_0 = safe_ctxt.input;
    if in_0.is_null() {
        return -1;
    }
    base = unsafe { (*in_0).cur.offset_from((*in_0).base) as i32 };
    if base < 0 {
        return -1;
    }
    if safe_ctxt.checkIndex > base as i64 {
        base = safe_ctxt.checkIndex as i32
    }
    let mut safe_in_0 = unsafe { &mut *in_0 };
    if (safe_in_0).buf.is_null() {
        buf = (safe_in_0).base;
        len = (safe_in_0).length
    } else {
        unsafe {
            buf = xmlBufContent((*(*in_0).buf).buffer as *const xmlBuf);
            len = xmlBufUse((*(*in_0).buf).buffer) as i32;
        }
    }
    /* take into account the sequence length */
    if third != 0 {
        len -= 2
    } else if next != 0 {
        len -= 1
    }
    let mut current_block_20: u64;
    while base < len {
        if unsafe { *buf.offset(base as isize) as i32 == first as i32 } {
            unsafe {
                if third as i32 != 0 {
                    if *buf.offset((base + 1 as i32) as isize) as i32 != next as i32
                        || *buf.offset((base + 2) as isize) as i32 != third as i32
                    {
                        current_block_20 = 2370887241019905314;
                    } else {
                        current_block_20 = 18386322304582297246;
                    }
                } else if next as i32 != 0 {
                    if *buf.offset((base + 1 as i32) as isize) as i32 != next as i32 {
                        current_block_20 = 2370887241019905314;
                    } else {
                        current_block_20 = 18386322304582297246;
                    }
                } else {
                    current_block_20 = 18386322304582297246;
                }
            }
            match current_block_20 {
                2370887241019905314 => {}
                _ => {
                    safe_ctxt.checkIndex = 0 as i64;

                    //#ifdef DEBUG_PUSH
                    match () {
                        #[cfg(HAVE_parser_DEBUG_PUSH)]
                        _ => unsafe {
                            if next as i32 == 0 {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: lookup \'%c\' found at %d\n\x00" as *const u8
                                        as *const i8,
                                    first as i32,
                                    base,
                                );
                            } else if third as i32 == 0 {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: lookup \'%c%c\' found at %d\n\x00" as *const u8
                                        as *const i8,
                                    first as i32,
                                    next as i32,
                                    base,
                                );
                            } else {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: lookup \'%c%c%c\' found at %d\n\x00" as *const u8
                                        as *const i8,
                                    first as i32,
                                    next as i32,
                                    third as i32,
                                    base,
                                );
                            }
                        },
                        #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                        _ => {}
                    };

                    //#endif

                    return unsafe {
                        (base as i64 - (*in_0).cur.offset_from((*in_0).base) as i64) as i32
                    };
                }
            }
        }
        base += 1
    }
    safe_ctxt.checkIndex = base as i64;

    match () {
        #[cfg(HAVE_parser_DEBUG_PUSH)]
        _ => unsafe {
            if next as i32 == 0 {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"PP: lookup \'%c\' failed\n\x00" as *const u8 as *const i8,
                    first as i32,
                );
            } else if third as i32 == 0 {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"PP: lookup \'%c%c\' failed\n\x00" as *const u8 as *const i8,
                    first as i32,
                    next as i32,
                );
            } else {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"PP: lookup \'%c%c%c\' failed\n\x00" as *const u8 as *const i8,
                    first as i32,
                    next as i32,
                    third as i32,
                );
            }
        },
        #[cfg(not(HAVE_parser_DEBUG_PUSH))]
        _ => {}
    };

    return -1;
}
/* *
* xmlParseGetLasts:
* @ctxt:  an XML parser context
* @lastlt:  pointer to store the last '<' from the input
* @lastgt:  pointer to store the last '>' from the input
*
* Lookup the last < and > in the current chunk
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
unsafe fn xmlParseGetLasts(
    mut ctxt: xmlParserCtxtPtr,
    mut lastlt: *mut *const xmlChar,
    mut lastgt: *mut *const xmlChar,
) {
    let mut tmp: *const xmlChar = 0 as *const xmlChar;
    if ctxt.is_null() || lastlt.is_null() || lastgt.is_null() {
        unsafe {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"Internal error: xmlParseGetLasts\n\x00" as *const u8 as *const i8,
            );
        }
        return;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if safe_ctxt.progressive != 0 && safe_ctxt.inputNr == 1 {
        unsafe {
            tmp = (*(*ctxt).input).end;
            tmp = tmp.offset(-1);
            while tmp >= (*(*ctxt).input).base && *tmp != '<' as u8 {
                tmp = tmp.offset(-1)
            }
            if tmp < (*(*ctxt).input).base {
                *lastlt = 0 as *const xmlChar;
                *lastgt = 0 as *const xmlChar
            } else {
                *lastlt = tmp;
                tmp = tmp.offset(1);
                while tmp < (*(*ctxt).input).end && *tmp != '>' as u8 {
                    if *tmp == '\'' as u8 {
                        tmp = tmp.offset(1);
                        while tmp < (*(*ctxt).input).end && *tmp != '\'' as u8 {
                            tmp = tmp.offset(1)
                        }
                        if tmp < (*(*ctxt).input).end {
                            tmp = tmp.offset(1)
                        }
                    } else if *tmp == '\"' as u8 {
                        tmp = tmp.offset(1);
                        while tmp < (*(*ctxt).input).end && *tmp != '\"' as u8 {
                            tmp = tmp.offset(1)
                        }
                        if tmp < (*(*ctxt).input).end {
                            tmp = tmp.offset(1)
                        }
                    } else {
                        tmp = tmp.offset(1)
                    }
                }
                if tmp < (*(*ctxt).input).end {
                    *lastgt = tmp
                } else {
                    tmp = *lastlt;
                    tmp = tmp.offset(-1);
                    while tmp >= (*(*ctxt).input).base && *tmp != '>' as u8 {
                        tmp = tmp.offset(-1)
                    }
                    if tmp >= (*(*ctxt).input).base {
                        *lastgt = tmp
                    } else {
                        *lastgt = 0 as *const xmlChar
                    }
                }
            }
        }
    } else {
        unsafe {
            *lastlt = 0 as *const xmlChar;
            *lastgt = 0 as *const xmlChar;
        }
    };
}
/* *
* xmlCheckCdataPush:
* @cur: pointer to the block of characters
* @len: length of the block in bytes
* @complete: 1 if complete CDATA block is passed in, 0 if partial block
*
* Check that the block of characters is okay as SCdata content [20]
*
* Returns the number of bytes to pass if okay, a negative index where an
*         UTF-8 error occurred otherwise
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
unsafe fn xmlCheckCdataPush(mut utf: *const xmlChar, mut len: i32, mut complete: i32) -> i32 {
    let mut ix: i32 = 0;
    let mut c: u8 = 0;
    let mut codepoint: i32 = 0;
    if utf.is_null() || len <= 0 {
        return 0;
    }
    ix = 0;
    while ix < len {
        /* string is 0-terminated */
        c = unsafe { *utf.offset(ix as isize) };
        if c as i32 & 0x80 == 0 {
            /* 1-byte code, starts with 10 */
            if c as i32 >= 0x20 || c as i32 == 0xa || c as i32 == 0xd || c as i32 == 0x9 {
                ix += 1
            } else {
                return -ix;
            }
        } else if c as i32 & 0xe0 == 0xc0 {
            /* 2-byte code, starts with 110 */
            if ix + 2 > len {
                return if complete != 0 { -ix } else { ix };
            }
            unsafe {
                if *utf.offset((ix + 1 as i32) as isize) as i32 & 0xc0 != 0x80 {
                    return -ix;
                }
                codepoint = (*utf.offset(ix as isize) as i32 & 0x1f) << 6;
                codepoint |= *utf.offset((ix + 1 as i32) as isize) as i32 & 0x3f;
            }
            if if codepoint < 0x100 {
                (0x9 <= codepoint && codepoint <= 0xa || codepoint == 0xd || 0x20 <= codepoint)
                    as i32
            } else {
                (0x100 <= codepoint && codepoint <= 0xd7ff
                    || 0xe000 <= codepoint && codepoint <= 0xfffd
                    || 0x10000 <= codepoint && codepoint <= 0x10ffff) as i32
            } == 0
            {
                return -ix;
            }
            ix += 2
        } else if c as i32 & 0xf0 == 0xe0 {
            /* 3-byte code, starts with 1110 */
            if ix + 3 > len {
                return if complete != 0 { -ix } else { ix };
            } /* unknown encoding */
            if unsafe {
                *utf.offset((ix + 1 as i32) as isize) as i32 & 0xc0 != 0x80
                    || *utf.offset((ix + 2) as isize) as i32 & 0xc0 != 0x80
            } {
                return -ix;
            }
            unsafe {
                codepoint = (*utf.offset(ix as isize) as i32 & 0xf as i32) << 12;
                codepoint |= (*utf.offset((ix + 1 as i32) as isize) as i32 & 0x3f) << 6;
                codepoint |= *utf.offset((ix + 2) as isize) as i32 & 0x3f;
            }
            if if codepoint < 0x100 {
                (0x9 <= codepoint && codepoint <= 0xa || codepoint == 0xd || 0x20 <= codepoint)
                    as i32
            } else {
                (0x100 <= codepoint && codepoint <= 0xd7ff
                    || 0xe000 <= codepoint && codepoint <= 0xfffd
                    || 0x10000 <= codepoint && codepoint <= 0x10ffff) as i32
            } == 0
            {
                return -ix;
            }
            ix += 3
        } else if c as i32 & 0xf8 == 0xf0 {
            /* 4-byte code, starts with 11110 */
            if ix + 4 > len {
                return if complete != 0 { -ix } else { ix };
            }
            unsafe {
                if *utf.offset((ix + 1 as i32) as isize) as i32 & 0xc0 != 0x80
                    || *utf.offset((ix + 2) as isize) as i32 & 0xc0 != 0x80
                    || *utf.offset((ix + 3) as isize) as i32 & 0xc0 != 0x80
                {
                    return -ix;
                }
                codepoint = (*utf.offset(ix as isize) as i32 & 0x7) << 18;
                codepoint |= (*utf.offset((ix + 1 as i32) as isize) as i32 & 0x3f) << 12;
                codepoint |= (*utf.offset((ix + 2) as isize) as i32 & 0x3f) << 6;
                codepoint |= *utf.offset((ix + 3) as isize) as i32 & 0x3f;
            }
            if if codepoint < 0x100 {
                (0x9 <= codepoint && codepoint <= 0xa || codepoint == 0xd || 0x20 <= codepoint)
                    as i32
            } else {
                (0x100 <= codepoint && codepoint <= 0xd7ff
                    || 0xe000 <= codepoint && codepoint <= 0xfffd
                    || 0x10000 <= codepoint && codepoint <= 0x10ffff) as i32
            } == 0
            {
                return -ix;
            }
            ix += 4
        } else {
            return -ix;
        }
    }
    return ix;
}
/* *
* xmlParseTryOrFinish:
* @ctxt:  an XML parser context
* @terminate:  last chunk indicator
*
* Try to progress on parsing
*
* Returns zero if no parsing was possible
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
unsafe fn xmlParseTryOrFinish(mut ctxt: xmlParserCtxtPtr, mut terminate: i32) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut current_block: u64;
    let mut ret: i32 = 0;
    let mut avail: i32 = 0;
    let mut tlen: i32 = 0;
    let mut cur: xmlChar = 0;
    let mut next: xmlChar = 0;
    let mut lastlt: *const xmlChar = 0 as *const xmlChar;
    let mut lastgt: *const xmlChar = 0 as *const xmlChar;
    if safe_ctxt.input.is_null() {
        return 0;
    }

    match () {
        #[cfg(HAVE_parser_DEBUG_PUSH)]
        _ => unsafe {
            match safe_ctxt.instate {
                -1 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try EOF\n\x00" as *const u8 as *const i8,
                    );
                }
                0 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try START\n\x00" as *const u8 as *const i8,
                    );
                }
                1 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try MISC\n\x00" as *const u8 as *const i8,
                    );
                }
                5 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try COMMENT\n\x00" as *const u8 as *const i8,
                    );
                }
                4 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try PROLOG\n\x00" as *const u8 as *const i8,
                    );
                }
                6 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try START_TAG\n\x00" as *const u8 as *const i8,
                    );
                }
                7 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try CONTENT\n\x00" as *const u8 as *const i8,
                    );
                }
                8 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try CDATA_SECTION\n\x00" as *const u8 as *const i8,
                    );
                }
                9 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try END_TAG\n\x00" as *const u8 as *const i8,
                    );
                }
                10 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try ENTITY_DECL\n\x00" as *const u8 as *const i8,
                    );
                }
                11 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try ENTITY_VALUE\n\x00" as *const u8 as *const i8,
                    );
                }
                12 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try ATTRIBUTE_VALUE\n\x00" as *const u8 as *const i8,
                    );
                }
                3 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try DTD\n\x00" as *const u8 as *const i8,
                    );
                }
                14 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try EPILOG\n\x00" as *const u8 as *const i8,
                    );
                }
                2 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try PI\n\x00" as *const u8 as *const i8,
                    );
                }
                15 => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: try IGNORE\n\x00" as *const u8 as *const i8,
                    );
                }
                _ => {}
            }
        },
        #[cfg(not(HAVE_parser_DEBUG_PUSH))]
        _ => {}
    };

    if !safe_ctxt.input.is_null()
        && unsafe { (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64 > 4096 as i64 }
    {
        xmlSHRINK(ctxt);
        safe_ctxt.checkIndex = 0 as i64
    }
    xmlParseGetLasts(ctxt, &mut lastlt, &mut lastgt);
    loop {
        if !(safe_ctxt.instate != XML_PARSER_EOF) {
            current_block = 1672565932838553232;
            break;
        }
        if safe_ctxt.errNo != XML_ERR_OK as i32 && safe_ctxt.disableSAX == 1 as i32 {
            return 0;
        }
        if safe_ctxt.input.is_null() {
            current_block = 1672565932838553232;
            break;
        }
        unsafe {
            if (*(*ctxt).input).buf.is_null() {
                avail = ((*(*ctxt).input).length as i64
                    - (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64)
                    as i32
            } else {
                /*
                 * If we are operating on converted input, try to flush
                 * remaining chars to avoid them stalling in the non-converted
                 * buffer. But do not do this in document start where
                 * encoding="..." may not have been read and we work on a
                 * guessed encoding.
                 */
                if (*ctxt).instate != XML_PARSER_START as i32
                    && !(*(*(*ctxt).input).buf).raw.is_null()
                    && xmlBufIsEmpty((*(*(*ctxt).input).buf).raw) == 0
                {
                    let mut base: size_t =
                        xmlBufGetInputBase((*(*(*ctxt).input).buf).buffer, (*ctxt).input);
                    let mut current: size_t =
                        (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64 as size_t;
                    xmlParserInputBufferPush(
                        (*(*ctxt).input).buf,
                        0,
                        b"\x00" as *const u8 as *const i8,
                    );
                    xmlBufSetInputBaseCur(
                        (*(*(*ctxt).input).buf).buffer,
                        (*ctxt).input,
                        base,
                        current,
                    );
                }
                avail = xmlBufUse((*(*(*ctxt).input).buf).buffer).wrapping_sub(
                    (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64 as u64,
                ) as i32
            }
        }
        if avail < 1 as i32 {
            current_block = 1672565932838553232;
            break;
        }
        match safe_ctxt.instate {
            -1 => {
                current_block = 1672565932838553232;
                break;
            }
            0 => {
                if safe_ctxt.charset == XML_CHAR_ENCODING_NONE {
                    let mut start: [xmlChar; 4] = [0; 4];
                    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
                    /*
                     * Very first chars read from the document flow.
                     */
                    if avail < 4 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    unsafe {
                        /*
                         * Get the 4 first bytes and decode the charset
                         * if enc != XML_CHAR_ENCODING_NONE
                         * plug some encoding conversion routines,
                         * else xmlSwitchEncoding will set to (default)
                         * UTF8.
                         */
                        start[0] = *(*(*ctxt).input).cur;
                        start[1] = *(*(*ctxt).input).cur.offset(1);
                        start[2] = *(*(*ctxt).input).cur.offset(2);
                        start[3] = *(*(*ctxt).input).cur.offset(3);
                    }
                    enc = xmlDetectCharEncoding_safe(start.as_mut_ptr(), 4);
                    xmlSwitchEncoding_safe(ctxt, enc);
                } else {
                    if avail < 2 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    unsafe {
                        cur = *(*(*ctxt).input).cur.offset(0);
                        next = *(*(*ctxt).input).cur.offset(1);
                    }
                    if cur as i32 == 0 {
                        unsafe {
                            if !(*ctxt).sax.is_null() && (*(*ctxt).sax).setDocumentLocator.is_some()
                            {
                                (*(*ctxt).sax)
                                    .setDocumentLocator
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    __xmlDefaultSAXLocator(),
                                );
                            }
                        }
                        xmlFatalErr(ctxt, XML_ERR_DOCUMENT_EMPTY, 0 as *const i8);
                        unsafe {
                            xmlHaltParser(ctxt);
                        }

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => unsafe {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: entering EOF\n\x00" as *const u8 as *const i8,
                                );
                            },
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };

                        unsafe {
                            if !(*ctxt).sax.is_null() && (*(*ctxt).sax).endDocument.is_some() {
                                (*(*ctxt).sax)
                                    .endDocument
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData
                                );
                            }
                        }
                        current_block = 1672565932838553232;
                        break;
                    } else if cur == '<' as u8 && next == '?' as u8 {
                        /* PI or XML decl */
                        if avail < 5 {
                            return ret;
                        }
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '?' as u8 as xmlChar,
                                '>' as u8 as xmlChar,
                                0 as xmlChar,
                            ) < 0
                        {
                            return ret;
                        }
                        unsafe {
                            if !(*ctxt).sax.is_null() && (*(*ctxt).sax).setDocumentLocator.is_some()
                            {
                                (*(*ctxt).sax)
                                    .setDocumentLocator
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    __xmlDefaultSAXLocator(),
                                );
                            }
                            if *(*(*ctxt).input).cur.offset(2) == 'x' as u8
                                && *(*(*ctxt).input).cur.offset(3) == 'm' as u8
                                && *(*(*ctxt).input).cur.offset(4) == 'l' as u8
                                && (*(*(*ctxt).input).cur.offset(5) as i32 == 0x20
                                    || 0x9 <= *(*(*ctxt).input).cur.offset(5) as i32
                                        && *(*(*ctxt).input).cur.offset(5) as i32 <= 0xa
                                    || *(*(*ctxt).input).cur.offset(5) as i32 == 0xd)
                            {
                                ret += 5;

                                match () {
                                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                                    _ => {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: Parsing XML Decl\n\x00" as *const u8 as *const i8,
                                        );
                                    }
                                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                    _ => {}
                                };
                                xmlParseXMLDecl(ctxt);
                                if (*ctxt).errNo == XML_ERR_UNSUPPORTED_ENCODING as i32 {
                                    /*
                                     * The XML REC instructs us to stop parsing right
                                     * here
                                     */
                                    xmlHaltParser(ctxt);
                                    return 0;
                                }
                                (*ctxt).standalone = (*(*ctxt).input).standalone;
                                if (*ctxt).encoding.is_null()
                                    && !(*(*ctxt).input).encoding.is_null()
                                {
                                    (*ctxt).encoding = xmlStrdup((*(*ctxt).input).encoding)
                                }
                                if !(*ctxt).sax.is_null()
                                    && (*(*ctxt).sax).startDocument.is_some()
                                    && (*ctxt).disableSAX == 0
                                {
                                    (*(*ctxt).sax)
                                        .startDocument
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData
                                    );
                                }
                                (*ctxt).instate = XML_PARSER_MISC;

                                match () {
                                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                                    _ => {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: entering MISC\n\x00" as *const u8 as *const i8,
                                        );
                                    }
                                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                    _ => {}
                                };
                            } else {
                                (*ctxt).version =
                                    xmlCharStrdup_safe(b"1.0\x00" as *const u8 as *const i8);
                                if !(*ctxt).sax.is_null()
                                    && (*(*ctxt).sax).startDocument.is_some()
                                    && (*ctxt).disableSAX == 0
                                {
                                    (*(*ctxt).sax)
                                        .startDocument
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData
                                    );
                                }
                                (*ctxt).instate = XML_PARSER_MISC;

                                match () {
                                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                                    _ => {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: entering MISC\n\x00" as *const u8 as *const i8,
                                        );
                                    }
                                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                    _ => {}
                                };
                            }
                        }
                    } else {
                        unsafe {
                            if !(*ctxt).sax.is_null() && (*(*ctxt).sax).setDocumentLocator.is_some()
                            {
                                (*(*ctxt).sax)
                                    .setDocumentLocator
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData,
                                    __xmlDefaultSAXLocator(),
                                );
                            }
                        }
                        safe_ctxt.version =
                            xmlCharStrdup_safe(b"1.0\x00" as *const u8 as *const i8);
                        if safe_ctxt.version.is_null() {
                            unsafe {
                                xmlErrMemory(ctxt, 0 as *const i8);
                            }
                        } else {
                            unsafe {
                                if !(*ctxt).sax.is_null()
                                    && (*(*ctxt).sax).startDocument.is_some()
                                    && (*ctxt).disableSAX == 0
                                {
                                    (*(*ctxt).sax)
                                        .startDocument
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData
                                    );
                                }
                                (*ctxt).instate = XML_PARSER_MISC;
                            }
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => unsafe {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: entering MISC\n\x00" as *const u8 as *const i8,
                                    );
                                },
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };
                        }
                    }
                }
            }
            6 => {
                let mut name: *const xmlChar = 0 as *const xmlChar;
                let mut prefix: *const xmlChar = 0 as *const xmlChar;
                let mut URI: *const xmlChar = 0 as *const xmlChar;
                let mut line: i32 = unsafe { (*(*ctxt).input).line };
                let mut nsNr: i32 = safe_ctxt.nsNr;
                if avail < 2 && safe_ctxt.inputNr == 1 {
                    current_block = 1672565932838553232;
                    break;
                }
                cur = unsafe { *(*(*ctxt).input).cur.offset(0) };
                if cur != '<' as u8 {
                    xmlFatalErr(ctxt, XML_ERR_DOCUMENT_EMPTY, 0 as *const i8);
                    unsafe {
                        xmlHaltParser(ctxt);
                        if !(*ctxt).sax.is_null() && (*(*ctxt).sax).endDocument.is_some() {
                            (*(*ctxt).sax)
                                .endDocument
                                .expect("non-null function pointer")(
                                (*ctxt).userData
                            );
                        }
                    }
                    current_block = 1672565932838553232;
                    break;
                } else {
                    if terminate == 0 {
                        if safe_ctxt.progressive != 0 {
                            /* > can be found unescaped in attribute values */
                            if lastgt.is_null() || unsafe { (*(*ctxt).input).cur } >= lastgt {
                                current_block = 1672565932838553232;
                                break;
                            }
                        } else if xmlParseLookupSequence(
                            ctxt,
                            '>' as u8 as xmlChar,
                            0 as xmlChar,
                            0 as xmlChar,
                        ) < 0
                        {
                            current_block = 1672565932838553232;
                            break;
                        }
                    }
                    if safe_ctxt.spaceNr == 0 || unsafe { *(*ctxt).space } == -2 {
                        spacePush(ctxt, -1);
                    } else {
                        spacePush(ctxt, unsafe { *(*ctxt).space });
                    }

                    match () {
                        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
                        _ => {
                            if safe_ctxt.sax2 != 0 {
                                /* LIBXML_SAX1_ENABLED */
                                name = xmlParseStartTag2(ctxt, &mut prefix, &mut URI, &mut tlen)
                            } else {
                                name = xmlParseStartTag(ctxt)
                            }
                        }
                        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
                        _ => {
                            name = xmlParseStartTag(ctxt);
                        }
                    };

                    /* LIBXML_SAX1_ENABLED */
                    if safe_ctxt.instate == XML_PARSER_EOF {
                        current_block = 1672565932838553232;
                        break;
                    }
                    if name.is_null() {
                        spacePop(ctxt);
                        unsafe {
                            xmlHaltParser(ctxt);
                            if !safe_ctxt.sax.is_null() && (*(*ctxt).sax).endDocument.is_some() {
                                (*(*ctxt).sax)
                                    .endDocument
                                    .expect("non-null function pointer")(
                                    (*ctxt).userData
                                );
                            }
                        }
                        current_block = 1672565932838553232;
                        break;
                    } else {
                        /*
                         * [ VC: Root Element Type ]
                         * The Name in the document type declaration must match
                         * the element type of the root element.
                         */

                        match () {
                            #[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
                            _ => {
                                if safe_ctxt.validate != 0
                                    && safe_ctxt.wellFormed != 0
                                    && !safe_ctxt.myDoc.is_null()
                                    && !safe_ctxt.node.is_null()
                                    && safe_ctxt.node == unsafe { (*(*ctxt).myDoc).children }
                                {
                                    safe_ctxt.valid &=
                                        xmlValidateRoot_safe(&mut safe_ctxt.vctxt, safe_ctxt.myDoc)
                                }
                            }
                            #[cfg(not(HAVE_parser_LIBXML_VALID_ENABLED))]
                            _ => {}
                        };

                        /* LIBXML_VALID_ENABLED */
                        /*
                         * Check for an Empty Element.
                         */
                        //@todo 削减unsafe范围
                        unsafe {
                            if *(*(*ctxt).input).cur == '/' as u8
                                && *(*(*ctxt).input).cur.offset(1) == '>' as u8
                            {
                                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(2);
                                (*(*ctxt).input).col += 2;
                                if *(*(*ctxt).input).cur as i32 == 0 {
                                    xmlParserInputGrow_safe((*ctxt).input, 250);
                                }
                                if (*ctxt).sax2 != 0 {
                                    if !(*ctxt).sax.is_null()
                                        && (*(*ctxt).sax).endElementNs.is_some()
                                        && (*ctxt).disableSAX == 0
                                    {
                                        (*(*ctxt).sax)
                                            .endElementNs
                                            .expect("non-null function pointer")(
                                            (*ctxt).userData,
                                            name,
                                            prefix,
                                            URI,
                                        );
                                    }
                                    if (*ctxt).nsNr - nsNr > 0 {
                                        nsPop(ctxt, (*ctxt).nsNr - nsNr);
                                    }
                                } else {
                                    match () {
                                        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
                                        _ => {
                                            if !(*ctxt).sax.is_null()
                                                && (*(*ctxt).sax).endElement.is_some()
                                                && (*ctxt).disableSAX == 0
                                            {
                                                (*(*ctxt).sax)
                                                    .endElement
                                                    .expect("non-null function pointer")(
                                                    (*ctxt).userData,
                                                    name,
                                                );
                                            }
                                        }
                                        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
                                        _ => {}
                                    };
                                }

                                if (*ctxt).instate == XML_PARSER_EOF {
                                    current_block = 1672565932838553232;
                                    break;
                                }
                                spacePop(ctxt);
                                if (*ctxt).nameNr == 0 {
                                    (*ctxt).instate = XML_PARSER_EPILOG
                                } else {
                                    (*ctxt).instate = XML_PARSER_CONTENT
                                }
                                (*ctxt).progressive = 1
                            } else {
                                if *(*(*ctxt).input).cur == '>' as u8 {
                                    xmlNextChar_safe(ctxt);
                                } else {
                                    xmlFatalErrMsgStr(
                                        ctxt,
                                        XML_ERR_GT_REQUIRED,
                                        b"Couldn\'t find end of Start Tag %s\n\x00" as *const u8
                                            as *const i8,
                                        name,
                                    );
                                    nodePop(ctxt);
                                    spacePop(ctxt);
                                }
                                nameNsPush(ctxt, name, prefix, URI, line, (*ctxt).nsNr - nsNr);
                                (*ctxt).instate = XML_PARSER_CONTENT;
                                (*ctxt).progressive = 1 as i32
                            }
                        }
                    }
                }
            }
            7 => {
                let mut test: *const xmlChar = 0 as *const xmlChar;
                let mut cons: u32 = 0;
                if avail < 2 && safe_ctxt.inputNr == 1 as i32 {
                    current_block = 1672565932838553232;
                    break;
                }
                unsafe {
                    cur = *(*(*ctxt).input).cur.offset(0);
                    next = *(*(*ctxt).input).cur.offset(1);
                    test = (*(*ctxt).input).cur;
                    cons = (*(*ctxt).input).consumed as u32;
                }
                if cur == '<' as u8 && next == '/' as u8 {
                    safe_ctxt.instate = XML_PARSER_END_TAG
                } else {
                    if cur == '<' as u8 && next == '?' as u8 {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '?' as xmlChar,
                                '>' as xmlChar,
                                0 as xmlChar,
                            ) < 0
                        {
                            safe_ctxt.progressive = XML_PARSER_PI as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            xmlParsePI(ctxt);
                            safe_ctxt.instate = XML_PARSER_CONTENT;
                            safe_ctxt.progressive = 1
                        }
                    } else if cur == '<' as u8 && next != '!' as u8 {
                        safe_ctxt.instate = XML_PARSER_START_TAG;
                        continue;
                    } else if cur == '<' as u8
                        && next == '!' as u8
                        && unsafe {
                            *(*(*ctxt).input).cur.offset(2) == '-' as u8
                                && *(*(*ctxt).input).cur.offset(3) == '-' as u8
                        }
                    {
                        let mut term: i32 = 0;
                        if avail < 4 {
                            current_block = 1672565932838553232;
                            break;
                        }
                        unsafe {
                            (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(4);
                        }
                        term = xmlParseLookupSequence(
                            ctxt,
                            '-' as xmlChar,
                            '-' as xmlChar,
                            '>' as xmlChar,
                        );
                        unsafe {
                            (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(-(4 as isize));
                        }
                        if terminate == 0 && term < 0 {
                            safe_ctxt.progressive = XML_PARSER_COMMENT as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            xmlParseComment(ctxt);
                            safe_ctxt.instate = XML_PARSER_CONTENT;
                            safe_ctxt.progressive = 1
                        }
                    } else if cur == '<' as u8
                        && unsafe {
                            *(*(*ctxt).input).cur.offset(1) == '!' as u8
                                && *(*(*ctxt).input).cur.offset(2) == '[' as u8
                                && *(*(*ctxt).input).cur.offset(3) == 'C' as u8
                                && *(*(*ctxt).input).cur.offset(4) == 'D' as u8
                                && *(*(*ctxt).input).cur.offset(5) == 'A' as u8
                                && *(*(*ctxt).input).cur.offset(6) == 'T' as u8
                                && *(*(*ctxt).input).cur.offset(7) == 'A' as u8
                                && *(*(*ctxt).input).cur.offset(8) == '[' as u8
                        }
                    {
                        unsafe {
                            (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(9);
                            (*(*ctxt).input).col += 9;
                            if *(*(*ctxt).input).cur as i32 == 0 {
                                xmlParserInputGrow_safe((*ctxt).input, 250);
                            }
                        }
                        safe_ctxt.instate = XML_PARSER_CDATA_SECTION;
                        continue;
                    } else {
                        if cur == '<' as u8 && next == '!' as u8 && avail < 9 {
                            current_block = 1672565932838553232;
                            break;
                        }
                        if cur == '&' as u8 {
                            if terminate == 0
                                && xmlParseLookupSequence(
                                    ctxt,
                                    ';' as xmlChar,
                                    0 as xmlChar,
                                    0 as xmlChar,
                                ) < 0
                            {
                                current_block = 1672565932838553232;
                                break;
                            }
                            xmlParseReference(ctxt);
                        } else {
                            /* LIBXML_SAX1_ENABLED */
                            /* TODO Avoid the extra copy, handle directly !!! */
                            /*
                             * Goal of the following test is:
                             *  - minimize calls to the SAX 'character' callback
                             *    when they are mergeable
                             *  - handle an problem for isBlank when we only parse
                             *    a sequence of blank chars and the next one is
                             *    not available to check against '<' presence.
                             *  - tries to homogenize the differences in SAX
                             *    callbacks between the push and pull versions
                             *    of the parser.
                             */
                            if safe_ctxt.inputNr == 1 as i32 && avail < 300 {
                                if terminate == 0 {
                                    if safe_ctxt.progressive != 0 {
                                        if lastlt.is_null()
                                            || unsafe { (*(*ctxt).input).cur > lastlt }
                                        {
                                            current_block = 1672565932838553232;
                                            break;
                                        }
                                    } else if xmlParseLookupSequence(
                                        ctxt,
                                        '<' as u8 as xmlChar,
                                        0 as xmlChar,
                                        0 as xmlChar,
                                    ) < 0
                                    {
                                        current_block = 1672565932838553232;
                                        break;
                                    }
                                }
                            }
                            safe_ctxt.checkIndex = 0 as i64;
                            xmlParseCharData(ctxt, 0);
                        }
                    }
                    unsafe {
                        if !(cons as u64 == (*(*ctxt).input).consumed
                            && test == (*(*ctxt).input).cur)
                        {
                            continue;
                        }
                    }
                    xmlFatalErr(
                        ctxt,
                        XML_ERR_INTERNAL_ERROR,
                        b"detected an error in element content\n\x00" as *const u8 as *const i8,
                    );
                    unsafe {
                        xmlHaltParser(ctxt);
                    }
                }
            }
            9 => {
                if avail < 2 {
                    current_block = 1672565932838553232;
                    break;
                }
                if terminate == 0 {
                    if safe_ctxt.progressive != 0 {
                        /* > can be found unescaped in attribute values */
                        if lastgt.is_null() || unsafe { (*(*ctxt).input).cur >= lastgt } {
                            current_block = 1672565932838553232;
                            break;
                        }
                    } else if xmlParseLookupSequence(
                        ctxt,
                        '>' as u8 as xmlChar,
                        0 as xmlChar,
                        0 as xmlChar,
                    ) < 0
                    {
                        current_block = 1672565932838553232;
                        break;
                    }
                }
                if safe_ctxt.sax2 != 0 {
                    unsafe {
                        xmlParseEndTag2(
                            ctxt,
                            &mut *(*ctxt).pushTab.offset(((*ctxt).nameNr - 1 as i32) as isize),
                        );
                    }
                    nameNsPop(ctxt);
                } else {
                    match () {
                        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
                        _ => {
                            xmlParseEndTag1(ctxt, 0);
                        }
                        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
                        _ => {}
                    };
                }
                /* LIBXML_SAX1_ENABLED */
                if !(safe_ctxt.instate == XML_PARSER_EOF) {
                    if safe_ctxt.nameNr == 0 {
                        safe_ctxt.instate = XML_PARSER_EPILOG
                    } else {
                        safe_ctxt.instate = XML_PARSER_CONTENT
                    }
                }
            }
            8 => {
                /*
                 * The Push mode need to have the SAX callback for
                 * cdataBlock merge back contiguous callbacks.
                 */
                let mut base_0: i32 = 0;
                base_0 = xmlParseLookupSequence(
                    ctxt,
                    ']' as u8 as xmlChar,
                    ']' as u8 as xmlChar,
                    '>' as u8 as xmlChar,
                );
                if base_0 < 0 {
                    if !(avail >= 300 + 2) {
                        current_block = 1672565932838553232;
                        break;
                    }
                    let mut tmp: i32 = 0;
                    tmp = unsafe { xmlCheckCdataPush((*(*ctxt).input).cur, 300, 0) };
                    if tmp < 0 {
                        tmp = -tmp;
                        unsafe {
                            (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(tmp as isize);
                        }
                        current_block = 473085638830652887;
                        break;
                    } else {
                        if !safe_ctxt.sax.is_null() && safe_ctxt.disableSAX == 0 {
                            unsafe {
                                if (*(*ctxt).sax).cdataBlock.is_some() {
                                    (*(*ctxt).sax)
                                        .cdataBlock
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        (*(*ctxt).input).cur,
                                        tmp,
                                    );
                                } else if (*(*ctxt).sax).characters.is_some() {
                                    (*(*ctxt).sax)
                                        .characters
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        (*(*ctxt).input).cur,
                                        tmp,
                                    );
                                }
                            }
                        }
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        let mut skipl: i32 = 0;
                        skipl = 0;
                        while skipl < tmp {
                            unsafe {
                                if *(*(*ctxt).input).cur == '\n' as u8 {
                                    (*(*ctxt).input).line += 1;
                                    (*(*ctxt).input).col = 1
                                } else {
                                    (*(*ctxt).input).col += 1
                                }
                                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(1);
                            }
                            skipl += 1
                        }
                        if unsafe { *(*(*ctxt).input).cur as i32 == 0 } {
                            xmlParserInputGrow_safe(safe_ctxt.input, 250);
                        }
                        safe_ctxt.checkIndex = 0 as i64;
                        current_block = 1672565932838553232;
                        break;
                    }
                } else {
                    let mut tmp_0: i32 = 0;
                    tmp_0 = unsafe { xmlCheckCdataPush((*(*ctxt).input).cur, base_0, 1 as i32) };
                    if tmp_0 < 0 || tmp_0 != base_0 {
                        tmp_0 = -tmp_0;
                        unsafe {
                            (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(tmp_0 as isize)
                        };
                        current_block = 473085638830652887;
                        break;
                    } else {
                        if !safe_ctxt.sax.is_null()
                            && base_0 == 0
                            && unsafe { (*(*ctxt).sax).cdataBlock.is_some() }
                            && safe_ctxt.disableSAX == 0
                        {
                            unsafe {
                                /*
                                 * Special case to provide identical behaviour
                                 * between pull and push parsers on enpty CDATA
                                 * sections
                                 */
                                if (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64
                                    >= 9
                                    && strncmp(
                                        &*(*(*ctxt).input).cur.offset(-(9 as i32) as isize)
                                            as *const xmlChar
                                            as *const i8,
                                        b"<![CDATA[\x00" as *const u8 as *const i8,
                                        9,
                                    ) == 0
                                {
                                    (*(*ctxt).sax)
                                        .cdataBlock
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        b"\x00" as *const u8 as *const i8 as *mut xmlChar,
                                        0,
                                    );
                                }
                            }
                        } else if !safe_ctxt.sax.is_null()
                            && base_0 > 0
                            && safe_ctxt.disableSAX == 0
                        {
                            unsafe {
                                if (*(*ctxt).sax).cdataBlock.is_some() {
                                    (*(*ctxt).sax)
                                        .cdataBlock
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        (*(*ctxt).input).cur,
                                        base_0,
                                    );
                                } else if (*(*ctxt).sax).characters.is_some() {
                                    (*(*ctxt).sax)
                                        .characters
                                        .expect("non-null function pointer")(
                                        (*ctxt).userData,
                                        (*(*ctxt).input).cur,
                                        base_0,
                                    );
                                }
                            }
                        }
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        let mut skipl_0: i32 = 0;
                        skipl_0 = 0;
                        while skipl_0 < base_0 + 3 {
                            unsafe {
                                if *(*(*ctxt).input).cur == '\n' as u8 {
                                    (*(*ctxt).input).line += 1;
                                    (*(*ctxt).input).col = 1
                                } else {
                                    (*(*ctxt).input).col += 1
                                }
                                (*(*ctxt).input).cur = (*(*ctxt).input).cur.offset(1);
                            }
                            skipl_0 += 1
                        }
                        if unsafe { *(*(*ctxt).input).cur as i32 == 0 } {
                            xmlParserInputGrow_safe(safe_ctxt.input, 250);
                        }
                        safe_ctxt.checkIndex = 0 as i64;
                        safe_ctxt.instate = XML_PARSER_CONTENT;

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => unsafe {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: entering CONTENT\n\x00" as *const u8 as *const i8,
                                );
                            },
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };
                    }
                }
            }
            1 => {
                xmlSkipBlankChars(ctxt);
                unsafe {
                    if (*(*ctxt).input).buf.is_null() {
                        avail = ((*(*ctxt).input).length as i64
                            - (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64)
                            as i32
                    } else {
                        avail = xmlBufUse((*(*(*ctxt).input).buf).buffer)
                            .wrapping_sub((*(*ctxt).input).cur.offset_from((*(*ctxt).input).base)
                                as i64 as u64) as i32
                    }
                    if avail < 2 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    cur = *(*(*ctxt).input).cur.offset(0);
                    next = *(*(*ctxt).input).cur.offset(1);
                    if cur == '<' as u8 && next == '?' as u8 {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '?' as xmlChar,
                                '>' as xmlChar,
                                0 as xmlChar,
                            ) < 0
                        {
                            (*ctxt).progressive = XML_PARSER_PI as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: Parsing PI\n\x00" as *const u8 as *const i8,
                                    );
                                }
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };

                            xmlParsePI(ctxt);
                            if (*ctxt).instate == XML_PARSER_EOF {
                                current_block = 1672565932838553232;
                                break;
                            }
                            (*ctxt).instate = XML_PARSER_MISC;
                            (*ctxt).progressive = 1 as i32;
                            (*ctxt).checkIndex = 0 as i64;
                        }
                    } else if cur == '<' as u8
                        && next == '!' as u8
                        && *(*(*ctxt).input).cur.offset(2) == '-' as u8
                        && *(*(*ctxt).input).cur.offset(3) == '-' as u8
                    {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '-' as u8 as xmlChar,
                                '-' as u8 as xmlChar,
                                '>' as u8 as xmlChar,
                            ) < 0
                        {
                            (*ctxt).progressive = XML_PARSER_COMMENT as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: Parsing Comment\n\x00" as *const u8 as *const i8,
                                    );
                                }
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };

                            xmlParseComment(ctxt);
                            if (*ctxt).instate == XML_PARSER_EOF {
                                current_block = 1672565932838553232;
                                break;
                            }
                            (*ctxt).instate = XML_PARSER_MISC;
                            (*ctxt).progressive = 1 as i32;
                            (*ctxt).checkIndex = 0 as i64
                        }
                    } else if cur == '<' as u8
                        && next == '!' as u8
                        && *(*(*ctxt).input).cur.offset(2) == 'D' as u8
                        && *(*(*ctxt).input).cur.offset(3) == 'O' as u8
                        && *(*(*ctxt).input).cur.offset(4) == 'C' as u8
                        && *(*(*ctxt).input).cur.offset(5) == 'T' as u8
                        && *(*(*ctxt).input).cur.offset(6) == 'Y' as u8
                        && *(*(*ctxt).input).cur.offset(7) == 'P' as u8
                        && *(*(*ctxt).input).cur.offset(8) == 'E' as u8
                    {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '>' as u8 as xmlChar,
                                0 as xmlChar,
                                0 as xmlChar,
                            ) < 0
                        {
                            safe_ctxt.progressive = XML_PARSER_DTD as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: Parsing internal subset\n\x00" as *const u8
                                            as *const i8,
                                    );
                                }
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };

                            safe_ctxt.inSubset = 1 as i32;
                            safe_ctxt.progressive = 0;
                            safe_ctxt.checkIndex = 0 as i64;
                            xmlParseDocTypeDecl(ctxt);
                            if safe_ctxt.instate == XML_PARSER_EOF {
                                current_block = 1672565932838553232;
                                break;
                            }
                            if *(*(*ctxt).input).cur == '[' as u8 {
                                safe_ctxt.instate = XML_PARSER_DTD;

                                match () {
                                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                                    _ => {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: entering DTD\n\x00" as *const u8 as *const i8,
                                        );
                                    }
                                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                    _ => {}
                                };
                            } else {
                                /*
                                 * Create and update the external subset.
                                 */
                                safe_ctxt.inSubset = 2;
                                if !safe_ctxt.sax.is_null()
                                    && safe_ctxt.disableSAX == 0
                                    && (*(*ctxt).sax).externalSubset.is_some()
                                {
                                    (*(*ctxt).sax)
                                        .externalSubset
                                        .expect("non-null function pointer")(
                                        safe_ctxt.userData,
                                        safe_ctxt.intSubName,
                                        safe_ctxt.extSubSystem,
                                        safe_ctxt.extSubURI,
                                    );
                                }
                                safe_ctxt.inSubset = 0;
                                xmlCleanSpecialAttr(ctxt);
                                safe_ctxt.instate = XML_PARSER_PROLOG;

                                match () {
                                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                                    _ => {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: entering PROLOG\n\x00" as *const u8 as *const i8,
                                        );
                                    }
                                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                    _ => {}
                                };
                            }
                        }
                    } else {
                        if cur == '<' as u8 && next == '!' as u8 && avail < 9 {
                            current_block = 1672565932838553232;
                            break;
                        }
                        safe_ctxt.instate = XML_PARSER_START_TAG;
                        safe_ctxt.progressive = XML_PARSER_START_TAG as i32;
                        xmlParseGetLasts(ctxt, &mut lastlt, &mut lastgt);

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: entering START_TAG\n\x00" as *const u8 as *const i8,
                                );
                            }
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };
                    }
                }
            }
            4 => {
                xmlSkipBlankChars(ctxt);
                unsafe {
                    if (*(*ctxt).input).buf.is_null() {
                        avail = ((*(*ctxt).input).length as i64
                            - (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64)
                            as i32
                    } else {
                        avail = xmlBufUse((*(*(*ctxt).input).buf).buffer)
                            .wrapping_sub((*(*ctxt).input).cur.offset_from((*(*ctxt).input).base)
                                as i64 as u64) as i32
                    }
                    if avail < 2 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    cur = *(*(*ctxt).input).cur.offset(0);
                    next = *(*(*ctxt).input).cur.offset(1);
                }
                if cur == '<' as u8 && next == '?' as u8 {
                    if terminate == 0
                        && xmlParseLookupSequence(
                            ctxt,
                            '?' as u8 as xmlChar,
                            '>' as u8 as xmlChar,
                            0 as xmlChar,
                        ) < 0
                    {
                        safe_ctxt.progressive = XML_PARSER_PI as i32;
                        current_block = 1672565932838553232;
                        break;
                    } else {
                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => unsafe {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: Parsing PI\n\x00" as *const u8 as *const i8,
                                );
                            },
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };

                        xmlParsePI(ctxt);
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        safe_ctxt.instate = XML_PARSER_PROLOG;
                        safe_ctxt.progressive = 1 as i32
                    }
                } else if cur == '<' as u8
                    && next == '!' as u8
                    && unsafe {
                        *(*(*ctxt).input).cur.offset(2) == '-' as u8
                            && *(*(*ctxt).input).cur.offset(3) == '-' as u8
                    }
                {
                    if terminate == 0
                        && xmlParseLookupSequence(
                            ctxt,
                            '-' as xmlChar,
                            '-' as xmlChar,
                            '>' as xmlChar,
                        ) < 0
                    {
                        safe_ctxt.progressive = XML_PARSER_COMMENT as i32;
                        current_block = 1672565932838553232;
                        break;
                    } else {
                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => unsafe {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: Parsing Comment\n\x00" as *const u8 as *const i8,
                                );
                            },
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };

                        xmlParseComment(ctxt);
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        safe_ctxt.instate = XML_PARSER_PROLOG;
                        safe_ctxt.progressive = 1 as i32
                    }
                } else {
                    if cur == '<' as u8 && next == '!' as u8 && avail < 4 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    safe_ctxt.instate = XML_PARSER_START_TAG;
                    if safe_ctxt.progressive == 0 {
                        safe_ctxt.progressive = XML_PARSER_START_TAG as i32
                    }
                    xmlParseGetLasts(ctxt, &mut lastlt, &mut lastgt);

                    match () {
                        #[cfg(HAVE_parser_DEBUG_PUSH)]
                        _ => unsafe {
                            (*__xmlGenericError()).expect("non-null function pointer")(
                                *__xmlGenericErrorContext(),
                                b"PP: entering START_TAG\n\x00" as *const u8 as *const i8,
                            );
                        },
                        #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                        _ => {}
                    };
                }
            }
            14 => {
                xmlSkipBlankChars(ctxt);
                unsafe {
                    if (*(*ctxt).input).buf.is_null() {
                        avail = ((*(*ctxt).input).length as i64
                            - (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i64)
                            as i32
                    } else {
                        avail = xmlBufUse((*(*(*ctxt).input).buf).buffer)
                            .wrapping_sub((*(*ctxt).input).cur.offset_from((*(*ctxt).input).base)
                                as i64 as u64) as i32
                    }
                    if avail < 2 {
                        current_block = 1672565932838553232;
                        break;
                    }
                    cur = *(*(*ctxt).input).cur.offset(0);
                    next = *(*(*ctxt).input).cur.offset(1);
                    if cur == '<' as u8 && next == '?' as u8 {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '?' as xmlChar,
                                '>' as xmlChar,
                                0 as xmlChar,
                            ) < 0
                        {
                            safe_ctxt.progressive = XML_PARSER_PI as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: Parsing PI\n\x00" as *const u8 as *const i8,
                                    );
                                }
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };

                            xmlParsePI(ctxt);
                            if safe_ctxt.instate == XML_PARSER_EOF {
                                current_block = 1672565932838553232;
                                break;
                            }
                            safe_ctxt.instate = XML_PARSER_EPILOG;
                            safe_ctxt.progressive = 1 as i32
                        }
                    } else if cur == '<' as u8
                        && next == '!' as u8
                        && *(*(*ctxt).input).cur.offset(2) == '-' as u8
                        && *(*(*ctxt).input).cur.offset(3) == '-' as u8
                    {
                        if terminate == 0
                            && xmlParseLookupSequence(
                                ctxt,
                                '-' as xmlChar,
                                '-' as xmlChar,
                                '>' as xmlChar,
                            ) < 0
                        {
                            safe_ctxt.progressive = XML_PARSER_COMMENT as i32;
                            current_block = 1672565932838553232;
                            break;
                        } else {
                            match () {
                                #[cfg(HAVE_parser_DEBUG_PUSH)]
                                _ => {
                                    (*__xmlGenericError()).expect("non-null function pointer")(
                                        *__xmlGenericErrorContext(),
                                        b"PP: Parsing Comment\n\x00" as *const u8 as *const i8,
                                    );
                                }
                                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                                _ => {}
                            };

                            xmlParseComment(ctxt);
                            if safe_ctxt.instate == XML_PARSER_EOF {
                                current_block = 1672565932838553232;
                                break;
                            }
                            safe_ctxt.instate = XML_PARSER_EPILOG;
                            safe_ctxt.progressive = 1
                        }
                    } else {
                        if cur == '<' as u8 && next == '!' as u8 && avail < 4 {
                            current_block = 1672565932838553232;
                            break;
                        }
                        xmlFatalErr(ctxt, XML_ERR_DOCUMENT_END, 0 as *const i8);
                        xmlHaltParser(ctxt);

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: entering EOF\n\x00" as *const u8 as *const i8,
                                );
                            }
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };

                        if !safe_ctxt.sax.is_null() && (*(*ctxt).sax).endDocument.is_some() {
                            (*(*ctxt).sax)
                                .endDocument
                                .expect("non-null function pointer")(
                                safe_ctxt.userData
                            );
                        }
                        current_block = 1672565932838553232;
                        break;
                    }
                }
            }
            3 => {
                /*
                 * Sorry but progressive parsing of the internal subset
                 * is not expected to be supported. We first check that
                 * the full content of the internal subset is available and
                 * the parsing is launched only at that point.
                 * Internal subset ends up with "']' S? '>'" in an unescaped
                 * section and not in a ']]>' sequence which are conditional
                 * sections (whoever argued to keep that crap in XML deserve
                 * a place in hell !).
                 */
                let mut base_1: i32 = 0;
                let mut i: i32 = 0;
                let mut buf: *mut xmlChar = 0 as *mut xmlChar;
                let mut quote: xmlChar = 0 as xmlChar;
                let mut use_0: size_t = 0;
                unsafe {
                    base_1 = (*(*ctxt).input).cur.offset_from((*(*ctxt).input).base) as i32;
                }
                if base_1 < 0 {
                    return 0;
                }
                if safe_ctxt.checkIndex > base_1 as i64 {
                    base_1 = safe_ctxt.checkIndex as i32
                }
                unsafe {
                    buf = xmlBufContent((*(*(*ctxt).input).buf).buffer as *const xmlBuf);
                    use_0 = xmlBufUse((*(*(*ctxt).input).buf).buffer);
                }
                's_1946: loop {
                    if !((base_1 as u32 as u64) < use_0) {
                        current_block = 10059826840140668507;
                        break;
                    }
                    if quote as i32 != 0 {
                        if unsafe { *buf.offset(base_1 as isize) as i32 == quote as i32 } {
                            quote = 0 as xmlChar
                        }
                    } else {
                        if quote as i32 == 0 && unsafe { *buf.offset(base_1 as isize) == '<' as u8 }
                        {
                            let mut found: i32 = 0;
                            /* special handling of comments */
                            if unsafe {
                                ((base_1 as u32).wrapping_add(4) as u64) < use_0
                                    && *buf.offset((base_1 + 1 as i32) as isize) == '!' as u8
                                    && *buf.offset((base_1 + 2) as isize) == '-' as u8
                                    && *buf.offset((base_1 + 3) as isize) == '-' as u8
                            } {
                                while ((base_1 as u32).wrapping_add(3 as u32) as u64) < use_0 {
                                    if unsafe {
                                        *buf.offset(base_1 as isize) == '-' as u8
                                            && *buf.offset((base_1 + 1 as i32) as isize)
                                                == '-' as u8
                                            && *buf.offset((base_1 + 2) as isize) == '>' as u8
                                    } {
                                        found = 1 as i32;
                                        base_1 += 2;
                                        break;
                                    } else {
                                        base_1 += 1
                                    }
                                }
                                if found == 0 {
                                    current_block = 10059826840140668507;
                                    break;
                                }
                                current_block = 16936879297222305916;
                            } else {
                                current_block = 9828016697359808143;
                            }
                        } else {
                            current_block = 9828016697359808143;
                        }
                        match current_block {
                            16936879297222305916 => {}
                            _ => {
                                unsafe {
                                    if *buf.offset(base_1 as isize) == '\"' as u8 {
                                        quote = '\"' as u8 as xmlChar
                                    } else if *buf.offset(base_1 as isize) == '\'' as u8 {
                                        quote = '\'' as u8 as xmlChar
                                    } else if *buf.offset(base_1 as isize) == ']' as u8 {
                                        if (base_1 as u32).wrapping_add(1 as i32 as u32) as u64
                                            >= use_0
                                        {
                                            current_block = 10059826840140668507;
                                            break;
                                        }
                                        if *buf.offset((base_1 + 1 as i32) as isize) == ']' as u8 {
                                            /* conditional crap, skip both ']' ! */
                                            base_1 += 1
                                        } else {
                                            i = 1;
                                            loop {
                                                if !(((base_1 as u32).wrapping_add(i as u32)
                                                    as u64)
                                                    < use_0)
                                                {
                                                    current_block = 10059826840140668507;
                                                    break 's_1946;
                                                }
                                                if *buf.offset((base_1 + i) as isize) == '>' as u8 {
                                                    current_block = 8979048619177278161;
                                                    break 's_1946;
                                                }
                                                if !(*buf.offset((base_1 + i) as isize) as i32
                                                    == 0x20
                                                    || 0x9
                                                        <= *buf.offset((base_1 + i) as isize)
                                                            as i32
                                                        && *buf.offset((base_1 + i) as isize)
                                                            as i32
                                                            <= 0xa
                                                    || *buf.offset((base_1 + i) as isize) as i32
                                                        == 0xd)
                                                {
                                                    break;
                                                }
                                                i += 1
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    base_1 += 1
                    /* for */
                }
                match current_block {
                    10059826840140668507 => {
                        /* for */
                        if quote as i32 == 0 {
                            safe_ctxt.checkIndex = base_1 as i64
                        } else {
                            safe_ctxt.checkIndex = 0 as i64
                        }

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => {
                                if next as i32 == 0 {
                                    unsafe {
                                        (*__xmlGenericError()).expect("non-null function pointer")(
                                            *__xmlGenericErrorContext(),
                                            b"PP: lookup of int subset end filed\n\x00" as *const u8
                                                as *const i8,
                                        );
                                    }
                                }
                            }
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };

                        current_block = 1672565932838553232;
                        break;
                    }
                    _ => {
                        safe_ctxt.checkIndex = 0 as i64;
                        xmlParseInternalSubset(ctxt);
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        safe_ctxt.inSubset = 2;
                        unsafe {
                            if !safe_ctxt.sax.is_null()
                                && safe_ctxt.disableSAX == 0
                                && (*(*ctxt).sax).externalSubset.is_some()
                            {
                                (*(*ctxt).sax)
                                    .externalSubset
                                    .expect("non-null function pointer")(
                                    safe_ctxt.userData,
                                    safe_ctxt.intSubName,
                                    safe_ctxt.extSubSystem,
                                    safe_ctxt.extSubURI,
                                );
                            }
                        }
                        safe_ctxt.inSubset = 0;
                        xmlCleanSpecialAttr(ctxt);
                        if safe_ctxt.instate == XML_PARSER_EOF {
                            current_block = 1672565932838553232;
                            break;
                        }
                        safe_ctxt.instate = XML_PARSER_PROLOG;
                        safe_ctxt.checkIndex = 0 as i64;

                        match () {
                            #[cfg(HAVE_parser_DEBUG_PUSH)]
                            _ => unsafe {
                                (*__xmlGenericError()).expect("non-null function pointer")(
                                    *__xmlGenericErrorContext(),
                                    b"PP: entering PROLOG\n\x00" as *const u8 as *const i8,
                                );
                            },
                            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                            _ => {}
                        };
                    }
                }
            }
            5 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == COMMENT\n\x00" as *const u8 as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_CONTENT;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering CONTENT\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            15 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == IGNORE\x00" as *const u8 as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_DTD;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering DTD\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            2 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == PI\n\x00" as *const u8 as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_CONTENT;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering CONTENT\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            10 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == ENTITY_DECL\n\x00" as *const u8 as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_DTD;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering DTD\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            11 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == ENTITY_VALUE\n\x00" as *const u8
                            as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_CONTENT;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering DTD\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            12 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == ATTRIBUTE_VALUE\n\x00" as *const u8
                            as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_START_TAG;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering START_TAG\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            13 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == SYSTEM_LITERAL\n\x00" as *const u8
                            as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_START_TAG;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering START_TAG\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            16 => {
                unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: internal error, state == PUBLIC_LITERAL\n\x00" as *const u8
                            as *const i8,
                    );
                }
                safe_ctxt.instate = XML_PARSER_START_TAG;

                match () {
                    #[cfg(HAVE_parser_DEBUG_PUSH)]
                    _ => unsafe {
                        (*__xmlGenericError()).expect("non-null function pointer")(
                            *__xmlGenericErrorContext(),
                            b"PP: entering START_TAG\n\x00" as *const u8 as *const i8,
                        );
                    },
                    #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                    _ => {}
                };
            }
            _ => {}
        }
    }
    match current_block {
        1672565932838553232 =>
        /*
         * We didn't found the end of the Internal subset
         */
        /*
         * Document parsing is done !
         */
        {
            match () {
                #[cfg(HAVE_parser_DEBUG_PUSH)]
                _ => unsafe {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: done %d\n\x00" as *const u8 as *const i8,
                        ret,
                    );
                },
                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                _ => {}
            };

            return ret;
        }
        _ => {
            let mut buffer: [i8; 150] = [0; 150];
            unsafe {
                snprintf(
                    buffer.as_mut_ptr(),
                    149,
                    b"Bytes: 0x%02X 0x%02X 0x%02X 0x%02X\n\x00" as *const u8 as *const i8,
                    *(*(*ctxt).input).cur.offset(0) as i32,
                    *(*(*ctxt).input).cur.offset(1) as i32,
                    *(*(*ctxt).input).cur.offset(2) as i32,
                    *(*(*ctxt).input).cur.offset(3) as i32,
                );

                __xmlErrEncoding(
                    ctxt,
                    XML_ERR_INVALID_CHAR,
                    b"Input is not proper UTF-8, indicate encoding !\n%s\x00" as *const u8
                        as *const i8,
                    buffer.as_mut_ptr() as *mut xmlChar,
                    0 as *const xmlChar,
                );
            }
            return 0;
        }
    };
}

/* *
* xmlParseCheckTransition:
* @ctxt:  an XML parser context
* @chunk:  a char array
* @size:  the size in byte of the chunk
*
* Check depending on the current parser state if the chunk given must be
* processed immediately or one need more data to advance on parsing.
*
* Returns -1 in case of error, 0 if the push is not needed and 1 if needed
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
unsafe fn xmlParseCheckTransition(
    mut ctxt: xmlParserCtxtPtr,
    mut chunk: *const i8,
    mut size: i32,
) -> i32 {
    if ctxt.is_null() || chunk.is_null() || size < 0 {
        return -1;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if safe_ctxt.instate == XML_PARSER_START_TAG as i32 {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    if safe_ctxt.progressive == XML_PARSER_COMMENT as i32 {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    if safe_ctxt.instate == XML_PARSER_CDATA_SECTION as i32 {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    if safe_ctxt.progressive == XML_PARSER_PI as i32 {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    if safe_ctxt.instate == XML_PARSER_END_TAG as i32 {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    if safe_ctxt.progressive == XML_PARSER_DTD as i32 || safe_ctxt.instate == XML_PARSER_DTD as i32
    {
        if unsafe { !memchr(chunk as *const (), '>' as i32, size as u64).is_null() } {
            return 1 as i32;
        }
        return 0;
    }
    return 1 as i32;
}

/* *
* xmlParseChunk:
* @ctxt:  an XML parser context
* @chunk:  an char array
* @size:  the size in byte of the chunk
* @terminate:  last chunk indicator
*
* Parse a Chunk of memory
*
* Returns zero if no error, the xmlParserErrors otherwise.
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
pub unsafe fn xmlParseChunk(
    mut ctxt: xmlParserCtxtPtr,
    mut chunk: *const i8,
    mut size: i32,
    mut terminate: i32,
) -> i32 {
    let mut end_in_lf: i32 = 0;
    let mut remain: i32 = 0;
    let mut old_avail: size_t = 0 as size_t;
    let mut avail: size_t = 0 as size_t;
    if ctxt.is_null() {
        return XML_ERR_INTERNAL_ERROR as i32;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if safe_ctxt.errNo != XML_ERR_OK as i32 && safe_ctxt.disableSAX == 1 as i32 {
        return safe_ctxt.errNo;
    }
    if safe_ctxt.instate == XML_PARSER_EOF {
        return -1;
    }
    if safe_ctxt.instate == XML_PARSER_START as i32 {
        unsafe {
            xmlDetectSAX2(ctxt);
        }
    }
    if size > 0
        && !chunk.is_null()
        && terminate == 0
        && unsafe { *chunk.offset((size - 1 as i32) as isize) as i32 == '\r' as i32 }
    {
        end_in_lf = 1;
        size -= 1
    }
    loop {
        if size > 0
            && !chunk.is_null()
            && !safe_ctxt.input.is_null()
            && unsafe { !(*safe_ctxt.input).buf.is_null() }
            && safe_ctxt.instate != XML_PARSER_EOF
        {
            let mut base: size_t = unsafe {
                xmlBufGetInputBase_safe((*(*safe_ctxt.input).buf).buffer, safe_ctxt.input)
            };
            let mut cur: size_t = unsafe {
                (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 as size_t
            };
            let mut res: i32 = 0;
            old_avail = unsafe { xmlBufUse((*(*safe_ctxt.input).buf).buffer) };
            /*
             * Specific handling if we autodetected an encoding, we should not
             * push more than the first line ... which depend on the encoding
             * And only push the rest once the final encoding was detected
             */
            unsafe {
                if safe_ctxt.instate == XML_PARSER_START as i32
                    && !safe_ctxt.input.is_null()
                    && !(*safe_ctxt.input).buf.is_null()
                    && !(*(*safe_ctxt.input).buf).encoder.is_null()
                {
                    let mut len: u32 = 45;
                    if !xmlStrcasestr(
                        (*(*(*safe_ctxt.input).buf).encoder).name as *mut xmlChar,
                        b"UTF-16\x00" as *const u8 as *const i8 as *mut xmlChar,
                    )
                    .is_null()
                        || !xmlStrcasestr(
                            (*(*(*safe_ctxt.input).buf).encoder).name as *mut xmlChar,
                            b"UTF16\x00" as *const u8 as *const i8 as *mut xmlChar,
                        )
                        .is_null()
                    {
                        len = 90 as u32
                    } else if !xmlStrcasestr(
                        (*(*(*safe_ctxt.input).buf).encoder).name as *mut xmlChar,
                        b"UCS-4\x00" as *const u8 as *const i8 as *mut xmlChar,
                    )
                    .is_null()
                        || !xmlStrcasestr(
                            (*(*(*safe_ctxt.input).buf).encoder).name as *mut xmlChar,
                            b"UCS4\x00" as *const u8 as *const i8 as *mut xmlChar,
                        )
                        .is_null()
                    {
                        len = 180 as u32
                    }
                    if (*(*safe_ctxt.input).buf).rawconsumed < len as u64 {
                        len = (len as u64).wrapping_sub((*(*safe_ctxt.input).buf).rawconsumed)
                            as u32 as u32
                    }
                    /*
                     * Change size for reading the initial declaration only
                     * if size is greater than len. Otherwise, memmove in xmlBufferAdd
                     * will blindly copy extra bytes from memory.
                     */
                    if size as u32 > len {
                        remain = (size as u32).wrapping_sub(len) as i32;
                        size = len as i32
                    } else {
                        remain = 0
                    }
                }
            }
            unsafe {
                res = xmlParserInputBufferPush_safe((*safe_ctxt.input).buf, size, chunk);
                xmlBufSetInputBaseCur_safe(
                    (*(*safe_ctxt.input).buf).buffer,
                    safe_ctxt.input,
                    base,
                    cur,
                );
            }
            if res < 0 {
                safe_ctxt.errNo = XML_PARSER_EOF;
                xmlHaltParser(ctxt);
                return XML_PARSER_EOF;
            }

            match () {
                #[cfg(HAVE_parser_DEBUG_PUSH)]
                _ => {
                    (*__xmlGenericError()).expect("non-null function pointer")(
                        *__xmlGenericErrorContext(),
                        b"PP: pushed %d\n\x00" as *const u8 as *const i8,
                        size,
                    );
                }
                #[cfg(not(HAVE_parser_DEBUG_PUSH))]
                _ => {}
            };
        } else if safe_ctxt.instate != XML_PARSER_EOF {
            if !safe_ctxt.input.is_null() && unsafe { !(*safe_ctxt.input).buf.is_null() } {
                let mut in_0: xmlParserInputBufferPtr = unsafe { (*safe_ctxt.input).buf };
                unsafe {
                    if !(*in_0).encoder.is_null()
                        && !(*in_0).buffer.is_null()
                        && !(*in_0).raw.is_null()
                    {
                        let mut nbchars: i32 = 0;
                        let mut base_0: size_t =
                            xmlBufGetInputBase((*in_0).buffer, safe_ctxt.input);
                        let mut current: size_t =
                            (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64
                                as size_t;
                        nbchars = xmlCharEncInput(in_0, terminate);
                        xmlBufSetInputBaseCur((*in_0).buffer, safe_ctxt.input, base_0, current);
                        if nbchars < 0 {
                            /* TODO 2.6.0 */
                            (*__xmlGenericError()).expect("non-null function pointer")(
                                *__xmlGenericErrorContext(),
                                b"xmlParseChunk: encoder error\n\x00" as *const u8 as *const i8,
                            );
                            xmlHaltParser(ctxt);
                            return XML_ERR_INVALID_ENCODING as i32;
                        }
                    }
                }
            }
        }
        if remain != 0 {
            unsafe {
                xmlParseTryOrFinish(ctxt, 0);
            }
        } else {
            unsafe {
                if !safe_ctxt.input.is_null() && !(*safe_ctxt.input).buf.is_null() {
                    avail = xmlBufUse((*(*safe_ctxt.input).buf).buffer)
                }
            }
            /*
             * Depending on the current state it may not be such
             * a good idea to try parsing if there is nothing in the chunk
             * which would be worth doing a parser state transition and we
             * need to wait for more data
             */
            unsafe {
                if terminate != 0
                    || avail > XML_MAX_TEXT_LENGTH
                    || old_avail == 0 as u64
                    || avail == 0 as u64
                    || xmlParseCheckTransition(
                        ctxt,
                        &*(*safe_ctxt.input).base.offset(old_avail as isize) as *const xmlChar
                            as *const i8,
                        avail.wrapping_sub(old_avail) as i32,
                    ) != 0
                {
                    xmlParseTryOrFinish(ctxt, terminate);
                }
            }
        }
        if safe_ctxt.instate == XML_PARSER_EOF {
            return safe_ctxt.errNo;
        }
        unsafe {
            if !safe_ctxt.input.is_null()
                && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as u64
                    > XML_MAX_TEXT_LENGTH
                    || (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as u64
                        > XML_MAX_TEXT_LENGTH)
                && safe_ctxt.options & XML_PARSE_HUGE as i32 == 0
            {
                xmlFatalErr(
                    ctxt,
                    XML_ERR_INTERNAL_ERROR,
                    b"Huge input lookup\x00" as *const u8 as *const i8,
                );
                xmlHaltParser(ctxt);
            }
        }
        if safe_ctxt.errNo != XML_ERR_OK as i32 && safe_ctxt.disableSAX == 1 as i32 {
            return safe_ctxt.errNo;
        }
        if !(remain != 0) {
            break;
        }
        unsafe {
            chunk = chunk.offset(size as isize);
        }
        size = remain;
        remain = 0
    }
    if end_in_lf == 1 as i32
        && !safe_ctxt.input.is_null()
        && unsafe { !(*safe_ctxt.input).buf.is_null() }
    {
        let mut base_1: size_t =
            unsafe { xmlBufGetInputBase((*(*safe_ctxt.input).buf).buffer, safe_ctxt.input) };
        let mut current_0: size_t =
            unsafe { (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 as size_t };
        unsafe {
            xmlParserInputBufferPush(
                (*safe_ctxt.input).buf,
                1 as i32,
                b"\r\x00" as *const u8 as *const i8,
            );
            xmlBufSetInputBaseCur(
                (*(*safe_ctxt.input).buf).buffer,
                safe_ctxt.input,
                base_1,
                current_0,
            );
        }
    }
    if terminate != 0 {
        /*
         * Check for termination
         */
        let mut cur_avail: i32 = 0;
        unsafe {
            if !safe_ctxt.input.is_null() {
                if (*safe_ctxt.input).buf.is_null() {
                    cur_avail = ((*safe_ctxt.input).length as i64
                        - (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64)
                        as i32
                } else {
                    cur_avail = xmlBufUse((*(*safe_ctxt.input).buf).buffer)
                        .wrapping_sub((*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base)
                            as i64 as u64) as i32
                }
            }
        }
        if safe_ctxt.instate != XML_PARSER_EOF && safe_ctxt.instate != XML_PARSER_EPILOG as i32 {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_DOCUMENT_END, 0 as *const i8);
            }
        }
        if safe_ctxt.instate == XML_PARSER_EPILOG as i32 && cur_avail > 0 {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_DOCUMENT_END, 0 as *const i8);
            }
        }
        if safe_ctxt.instate != XML_PARSER_EOF {
            if !safe_ctxt.sax.is_null() && unsafe { (*safe_ctxt.sax).endDocument.is_some() } {
                unsafe {
                    (*safe_ctxt.sax)
                        .endDocument
                        .expect("non-null function pointer")(safe_ctxt.userData);
                }
            }
        }
        safe_ctxt.instate = XML_PARSER_EOF
    }
    if safe_ctxt.wellFormed == 0 {
        return safe_ctxt.errNo as xmlParserErrors as i32;
    } else {
        return 0;
    };
}
/* ***********************************************************************
*									*
*		I/O front end functions to the parser			*
*									*
************************************************************************/
/* *
* xmlCreatePushParserCtxt:
* @sax:  a SAX handler
* @user_data:  The user data returned on SAX callbacks
* @chunk:  a pointer to an array of chars
* @size:  number of chars in the array
* @filename:  an optional file name or URI
*
* Create a parser context for using the XML parser in push mode.
* If @buffer and @size are non-NULL, the data is used to detect
* the encoding.  The remaining characters will be parsed so they
* don't need to be fed in again through xmlParseChunk.
* To allow content encoding detection, @size should be >= 4
* The value of @filename is used for fetching external entities
* and error/warning reports.
*
* Returns the new parser context or NULL
*/
#[cfg(HAVE_parser_LIBXML_PUSH_ENABLED)]
pub unsafe fn xmlCreatePushParserCtxt(
    mut sax: xmlSAXHandlerPtr,
    mut user_data: *mut (),
    mut chunk: *const i8,
    mut size: i32,
    mut filename: *const i8,
) -> xmlParserCtxtPtr {
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    let mut inputStream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    let mut buf: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
    /*
     * plug some encoding conversion routines
     */
    if !chunk.is_null() && size >= 4 {
        enc = xmlDetectCharEncoding_safe(chunk as *const xmlChar, size)
    }
    buf = xmlAllocParserInputBuffer_safe(enc);
    if buf.is_null() {
        return 0 as xmlParserCtxtPtr;
    }
    ctxt = xmlNewParserCtxt_safe();
    if ctxt.is_null() {
        xmlErrMemory(
            0 as xmlParserCtxtPtr,
            b"creating parser: out of memory\n\x00" as *const u8 as *const i8,
        );
        xmlFreeParserInputBuffer_safe(buf);
        return 0 as xmlParserCtxtPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    safe_ctxt.dictNames = 1 as i32;
    if !sax.is_null() {
        match () {
            #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
            _ => {
                if safe_ctxt.sax != __xmlDefaultSAXHandler_safe() as xmlSAXHandlerPtr {
                    /* LIBXML_SAX1_ENABLED */
                    xmlFree_safe(safe_ctxt.sax as *mut ());
                }
            }
            #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
            _ => {
                xmlFree_safe(safe_ctxt.sax as *mut ());
            }
        };

        safe_ctxt.sax = xmlMalloc_safe(size_of::<xmlSAXHandler>() as u64) as xmlSAXHandlerPtr;
        if safe_ctxt.sax.is_null() {
            xmlErrMemory(ctxt, 0 as *const i8);
            xmlFreeParserInputBuffer_safe(buf);
            xmlFreeParserCtxt_safe(ctxt);
            return 0 as xmlParserCtxtPtr;
        }
        unsafe {
            memset(
                safe_ctxt.sax as *mut (),
                0,
                size_of::<xmlSAXHandler>() as u64,
            );
            if (*sax).initialized == 0xdeedbeaf as u32 {
                memcpy(
                    safe_ctxt.sax as *mut (),
                    sax as *const (),
                    size_of::<xmlSAXHandler>() as u64,
                );
            } else {
                memcpy(
                    safe_ctxt.sax as *mut (),
                    sax as *const (),
                    size_of::<xmlSAXHandlerV1>() as u64,
                );
            }
        }
        if !user_data.is_null() {
            safe_ctxt.userData = user_data
        }
    }
    if filename.is_null() {
        safe_ctxt.directory = 0 as *mut i8
    } else {
        safe_ctxt.directory = xmlParserGetDirectory_safe(filename)
    }
    inputStream = xmlNewInputStream_safe(ctxt);
    if inputStream.is_null() {
        xmlFreeParserCtxt_safe(ctxt);
        xmlFreeParserInputBuffer_safe(buf);
        return 0 as xmlParserCtxtPtr;
    }
    let mut safe_inputStream = unsafe { &mut *inputStream };

    if filename.is_null() {
        safe_inputStream.filename = 0 as *const i8
    } else {
        safe_inputStream.filename = xmlCanonicPath_safe(filename as *const xmlChar) as *mut i8;
        if safe_inputStream.filename.is_null() {
            xmlFreeParserCtxt_safe(ctxt);
            xmlFreeParserInputBuffer_safe(buf);
            return 0 as xmlParserCtxtPtr;
        }
    }
    safe_inputStream.buf = buf;
    unsafe {
        xmlBufResetInput_safe((*safe_inputStream.buf).buffer, inputStream);
    }
    inputPush_safe(ctxt, inputStream);
    /*
     * If the caller didn't provide an initial 'chunk' for determining
     * the encoding, we set the context to XML_CHAR_ENCODING_NONE so
     * that it can be automatically determined later
     */
    if size == 0 || chunk.is_null() {
        safe_ctxt.charset = XML_CHAR_ENCODING_NONE
    } else if !safe_ctxt.input.is_null() && unsafe { !(*safe_ctxt.input).buf.is_null() } {
        let mut base: size_t =
            unsafe { xmlBufGetInputBase_safe((*(*safe_ctxt.input).buf).buffer, safe_ctxt.input) };
        let mut cur: size_t =
            unsafe { (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 as size_t };
        unsafe {
            xmlParserInputBufferPush_safe((*safe_ctxt.input).buf, size, chunk);
            xmlBufSetInputBaseCur_safe(
                (*(*safe_ctxt.input).buf).buffer,
                safe_ctxt.input,
                base,
                cur,
            );
        }

        match () {
            #[cfg(HAVE_parser_DEBUG_PUSH)]
            _ => {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"PP: pushed %d\n\x00" as *const u8 as *const i8,
                    size,
                );
            }
            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
            _ => {}
        };
    }

    if enc != XML_CHAR_ENCODING_NONE {
        xmlSwitchEncoding_safe(ctxt, enc);
    }
    return ctxt;
}

/* LIBXML_PUSH_ENABLED */
/* *
* xmlHaltParser:
* @ctxt:  an XML parser context
*
* Blocks further parser processing don't override error
* for internal use
*/
unsafe fn xmlHaltParser(mut ctxt: xmlParserCtxtPtr) {
    if ctxt.is_null() {
        return;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    safe_ctxt.instate = XML_PARSER_EOF;
    safe_ctxt.disableSAX = 1 as i32;
    while 1 < 2 {
        if (!(safe_ctxt.inputNr > 1 as i32)) {
            break;
        }
        xmlFreeInputStream_safe(inputPop_safe(ctxt));
    }
    if !safe_ctxt.input.is_null() {
        /*
         * in case there was a specific allocation deallocate before
         * overriding base
         */
        unsafe {
            if (*safe_ctxt.input).free.is_some() {
                (*safe_ctxt.input).free.expect("non-null function pointer")(
                    (*safe_ctxt.input).base as *mut xmlChar,
                );
                (*safe_ctxt.input).free = None
            }
            if !(*safe_ctxt.input).buf.is_null() {
                xmlFreeParserInputBuffer((*safe_ctxt.input).buf);
                (*safe_ctxt.input).buf = 0 as xmlParserInputBufferPtr
            }
            (*safe_ctxt.input).cur = b"\x00" as *const u8 as *const i8 as *mut xmlChar;
            (*safe_ctxt.input).length = 0;
            (*safe_ctxt.input).base = (*safe_ctxt.input).cur;
            (*safe_ctxt.input).end = (*safe_ctxt.input).cur
        }
    };
}
/* *
* xmlStopParser:
* @ctxt:  an XML parser context
*
* Blocks further parser processing
*/

pub unsafe fn xmlStopParser_parser(mut ctxt: xmlParserCtxtPtr) {
    if ctxt.is_null() {
        return;
    }
    xmlHaltParser(ctxt);
    let mut safe_ctxt = unsafe { &mut *ctxt };

    safe_ctxt.errNo = XML_ERR_USER_STOP as i32;
}
/* *
* xmlCreateIOParserCtxt:
* @sax:  a SAX handler
* @user_data:  The user data returned on SAX callbacks
* @ioread:  an I/O read function
* @ioclose:  an I/O close function
* @ioctx:  an I/O handler
* @enc:  the charset encoding if known
*
* Create a parser context for using the XML parser with an existing
* I/O stream
*
* Returns the new parser context or NULL
*/

pub unsafe fn xmlCreateIOParserCtxt(
    mut sax: xmlSAXHandlerPtr,
    mut user_data: *mut (),
    mut ioread: xmlInputReadCallback,
    mut ioclose: xmlInputCloseCallback,
    mut ioctx: *mut (),
    mut enc: xmlCharEncoding,
) -> xmlParserCtxtPtr {
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    let mut inputStream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    let mut buf: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    if ioread.is_none() {
        return 0 as xmlParserCtxtPtr;
    }
    buf = xmlParserInputBufferCreateIO_safe(ioread, ioclose, ioctx, enc);
    if buf.is_null() {
        if ioclose.is_some() {
            unsafe {
                ioclose.expect("non-null function pointer")(ioctx);
            }
        }
        return 0 as xmlParserCtxtPtr;
    }
    ctxt = xmlNewParserCtxt_safe();
    if ctxt.is_null() {
        xmlFreeParserInputBuffer_safe(buf);
        return 0 as xmlParserCtxtPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if !sax.is_null() {
        match () {
            #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
            _ => {
                if safe_ctxt.sax != __xmlDefaultSAXHandler_safe() as xmlSAXHandlerPtr {
                    /* LIBXML_SAX1_ENABLED */
                    xmlFree_safe(safe_ctxt.sax as *mut ());
                }
            }
            #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
            _ => {
                xmlFree_safe(safe_ctxt.sax as *mut ());
            }
        };

        safe_ctxt.sax = xmlMalloc_safe(size_of::<xmlSAXHandler>() as u64) as xmlSAXHandlerPtr;
        if safe_ctxt.sax.is_null() {
            xmlErrMemory(ctxt, 0 as *const i8);
            xmlFreeParserCtxt_safe(ctxt);
            return 0 as xmlParserCtxtPtr;
        }
        unsafe {
            memset(
                safe_ctxt.sax as *mut (),
                0,
                size_of::<xmlSAXHandler>() as u64,
            );
            if (*sax).initialized == 0xdeedbeaf as u32 {
                memcpy(
                    safe_ctxt.sax as *mut (),
                    sax as *const (),
                    size_of::<xmlSAXHandler>() as u64,
                );
            } else {
                memcpy(
                    safe_ctxt.sax as *mut (),
                    sax as *const (),
                    size_of::<xmlSAXHandlerV1>() as u64,
                );
            }
        }
        if !user_data.is_null() {
            safe_ctxt.userData = user_data
        }
    }
    inputStream = xmlNewIOInputStream_safe(ctxt, buf, enc);
    if inputStream.is_null() {
        xmlFreeParserCtxt_safe(ctxt);
        return 0 as xmlParserCtxtPtr;
    }
    inputPush_safe(ctxt, inputStream);
    return ctxt;
}

/* ***********************************************************************
*									*
*		Front ends when parsing a DTD				*
*									*
************************************************************************/
/* *
* xmlIOParseDTD:
* @sax:  the SAX handler block or NULL
* @input:  an Input Buffer
* @enc:  the charset encoding if known
*
* Load and parse a DTD
*
* Returns the resulting xmlDtdPtr or NULL in case of error.
* @input will be freed by the function in any case.
*/
#[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
pub fn xmlIOParseDTD(
    sax: xmlSAXHandlerPtr,
    input: xmlParserInputBufferPtr,
    mut enc: xmlCharEncoding,
) -> xmlDtdPtr {
    let mut ret: xmlDtdPtr = 0 as xmlDtdPtr;
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    let mut pinput: xmlParserInputPtr = 0 as xmlParserInputPtr;
    let mut start: [xmlChar; 4] = [0; 4];
    if input.is_null() {
        return 0 as xmlDtdPtr;
    }
    ctxt = unsafe { xmlNewParserCtxt_safe() };
    if ctxt.is_null() {
        unsafe {
            xmlFreeParserInputBuffer(input);
        }
        return 0 as xmlDtdPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    /* We are loading a DTD */
    safe_ctxt.options |= XML_PARSE_DTDLOAD as i32;
    /*
     * Set-up the SAX context
     */
    if !sax.is_null() {
        if !safe_ctxt.sax.is_null() {
            unsafe { xmlFree_safe(safe_ctxt.sax as *mut ()) };
        }
        safe_ctxt.sax = sax;
        safe_ctxt.userData = ctxt as *mut ()
    }
    unsafe {
        xmlDetectSAX2(ctxt);
        /*
         * generate a parser input from the I/O handler
         */
        pinput = xmlNewIOInputStream(ctxt, input, XML_CHAR_ENCODING_NONE);
    }
    if pinput.is_null() {
        if !sax.is_null() {
            safe_ctxt.sax = 0 as *mut _xmlSAXHandler
        }
        unsafe { xmlFreeParserInputBuffer_safe(input) };
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        return 0 as xmlDtdPtr;
    }
    /*
     * plug some encoding conversion routines here.
     */
    if unsafe { xmlPushInput(ctxt, pinput) < 0 } {
        if !sax.is_null() {
            safe_ctxt.sax = 0 as *mut _xmlSAXHandler
        }
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        return 0 as xmlDtdPtr;
    }
    if enc as i32 != XML_CHAR_ENCODING_NONE as i32 {
        unsafe { xmlSwitchEncoding_safe(ctxt, enc) };
    }
    let mut safe_pinput = unsafe { *pinput };

    safe_pinput.filename = 0 as *const i8;
    safe_pinput.line = 1;
    safe_pinput.col = 1;
    unsafe {
        safe_pinput.base = (*safe_ctxt.input).cur;
        safe_pinput.cur = (*safe_ctxt.input).cur;
    }
    safe_pinput.free = None;
    /*
     * let's parse that entity knowing it's an external subset.
     */
    safe_ctxt.inSubset = 2;
    unsafe {
        safe_ctxt.myDoc = xmlNewDoc(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar);
    }
    if safe_ctxt.myDoc.is_null() {
        unsafe {
            xmlErrMemory(ctxt, b"New Doc failed\x00" as *const u8 as *const i8);
        }
        return 0 as xmlDtdPtr;
    }
    unsafe {
        (*safe_ctxt.myDoc).properties = XML_DOC_INTERNAL as i32;
        (*safe_ctxt.myDoc).extSubset = xmlNewDtd(
            safe_ctxt.myDoc,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
        );
        if enc as i32 == XML_CHAR_ENCODING_NONE as i32
            && (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 >= 4
        {
            /*
             * Get the 4 first bytes and decode the charset
             * if enc != XML_CHAR_ENCODING_NONE
             * plug some encoding conversion routines.
             */
            start[0] = *(*safe_ctxt.input).cur;
            start[1] = *(*safe_ctxt.input).cur.offset(1);
            start[2] = *(*safe_ctxt.input).cur.offset(2);
            start[3] = *(*safe_ctxt.input).cur.offset(3);
            enc = xmlDetectCharEncoding(start.as_mut_ptr(), 4);
            if enc as i32 != XML_CHAR_ENCODING_NONE as i32 {
                xmlSwitchEncoding(ctxt, enc);
            }
        }

        xmlParseExternalSubset(
            ctxt,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
        );
    }
    if !safe_ctxt.myDoc.is_null() {
        if safe_ctxt.wellFormed != 0 {
            unsafe {
                ret = (*safe_ctxt.myDoc).extSubset;
                (*safe_ctxt.myDoc).extSubset = 0 as *mut _xmlDtd;
                if !ret.is_null() {
                    let mut tmp: xmlNodePtr = 0 as *mut xmlNode;
                    (*ret).doc = 0 as *mut _xmlDoc;
                    tmp = (*ret).children;
                    while !tmp.is_null() {
                        (*tmp).doc = 0 as *mut _xmlDoc;
                        tmp = (*tmp).next
                    }
                }
            }
        } else {
            ret = 0 as xmlDtdPtr
        }
        unsafe { xmlFreeDoc_safe(safe_ctxt.myDoc) };
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    unsafe { xmlFreeParserCtxt_safe(ctxt) };
    return ret;
}

/* *
* xmlSAXParseDTD:
* @sax:  the SAX handler block
* @ExternalID:  a NAME* containing the External ID of the DTD
* @SystemID:  a NAME* containing the URL to the DTD
*
* Load and parse an external subset.
*
* Returns the resulting xmlDtdPtr or NULL in case of error.
*/
#[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
pub fn xmlSAXParseDTD(
    sax: xmlSAXHandlerPtr,
    ExternalID: *const xmlChar,
    SystemID: *const xmlChar,
) -> xmlDtdPtr {
    let mut ret: xmlDtdPtr = 0 as xmlDtdPtr;
    let ctxt: xmlParserCtxtPtr;
    let mut input: xmlParserInputPtr = 0 as xmlParserInputPtr;
    let enc: xmlCharEncoding;
    let mut systemIdCanonic: *mut xmlChar = 0 as *mut xmlChar;
    if ExternalID.is_null() && SystemID.is_null() {
        return 0 as xmlDtdPtr;
    }
    ctxt = unsafe { xmlNewParserCtxt_safe() };
    if ctxt.is_null() {
        return 0 as xmlDtdPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    /* We are loading a DTD */
    safe_ctxt.options |= XML_PARSE_DTDLOAD as i32;
    /*
     * Set-up the SAX context
     */
    if !sax.is_null() {
        if !safe_ctxt.sax.is_null() {
            unsafe { xmlFree_safe(safe_ctxt.sax as *mut ()) };
        }
        safe_ctxt.sax = sax;
        safe_ctxt.userData = ctxt as *mut ()
    }
    /*
     * Canonicalise the system ID
     */
    systemIdCanonic = unsafe { xmlCanonicPath_safe(SystemID) };
    if !SystemID.is_null() && systemIdCanonic.is_null() {
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        return 0 as xmlDtdPtr;
    }
    /*
     * Ask the Entity resolver to load the damn thing
     */
    unsafe {
        if !safe_ctxt.sax.is_null() && (*safe_ctxt.sax).resolveEntity.is_some() {
            input = (*safe_ctxt.sax)
                .resolveEntity
                .expect("non-null function pointer")(
                safe_ctxt.userData,
                ExternalID,
                systemIdCanonic,
            )
        }
    }
    if input.is_null() {
        if !sax.is_null() {
            safe_ctxt.sax = 0 as *mut _xmlSAXHandler
        }
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        if !systemIdCanonic.is_null() {
            unsafe { xmlFree_safe(systemIdCanonic as *mut ()) };
        }
        return 0 as xmlDtdPtr;
    }
    /*
     * plug some encoding conversion routines here.
     */
    if unsafe { xmlPushInput(ctxt, input) < 0 } {
        if !sax.is_null() {
            safe_ctxt.sax = 0 as *mut _xmlSAXHandler
        }
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        if !systemIdCanonic.is_null() {
            unsafe { xmlFree_safe(systemIdCanonic as *mut ()) };
        }
        return 0 as xmlDtdPtr;
    }
    unsafe {
        if (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 >= 4 {
            enc = xmlDetectCharEncoding((*safe_ctxt.input).cur, 4);
            xmlSwitchEncoding(ctxt, enc);
        }
    }
    unsafe {
        if (*input).filename.is_null() {
            (*input).filename = systemIdCanonic as *mut i8
        } else {
            xmlFree_safe(systemIdCanonic as *mut ());
        }
        (*input).line = 1;
        (*input).col = 1;
        (*input).base = (*safe_ctxt.input).cur;
        (*input).cur = (*safe_ctxt.input).cur;
        (*input).free = None;
    }
    /*
     * let's parse that entity knowing it's an external subset.
     */
    safe_ctxt.inSubset = 2;
    unsafe {
        safe_ctxt.myDoc = xmlNewDoc(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar);
    }
    if safe_ctxt.myDoc.is_null() {
        unsafe {
            xmlErrMemory(ctxt, b"New Doc failed\x00" as *const u8 as *const i8);
        }
        if !sax.is_null() {
            safe_ctxt.sax = 0 as *mut _xmlSAXHandler
        }
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        return 0 as xmlDtdPtr;
    }
    unsafe {
        (*safe_ctxt.myDoc).properties = XML_DOC_INTERNAL as i32;
        (*safe_ctxt.myDoc).extSubset = xmlNewDtd(
            safe_ctxt.myDoc,
            b"none\x00" as *const u8 as *const i8 as *mut xmlChar,
            ExternalID,
            SystemID,
        );
    }
    unsafe {
        xmlParseExternalSubset(ctxt, ExternalID, SystemID);
        if !safe_ctxt.myDoc.is_null() {
            if safe_ctxt.wellFormed != 0 {
                ret = (*safe_ctxt.myDoc).extSubset;
                (*safe_ctxt.myDoc).extSubset = 0 as *mut _xmlDtd;
                if !ret.is_null() {
                    let mut tmp: xmlNodePtr = 0 as *mut xmlNode;
                    (*ret).doc = 0 as *mut _xmlDoc;
                    tmp = (*ret).children;
                    while !tmp.is_null() {
                        (*tmp).doc = 0 as *mut _xmlDoc;
                        tmp = (*tmp).next
                    }
                }
            } else {
                ret = 0 as xmlDtdPtr
            }
            xmlFreeDoc_safe(safe_ctxt.myDoc);
            safe_ctxt.myDoc = 0 as xmlDocPtr
        }
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    unsafe { xmlFreeParserCtxt_safe(ctxt) };
    return ret;
}
/* *
* xmlParseDTD:
* @ExternalID:  a NAME* containing the External ID of the DTD
* @SystemID:  a NAME* containing the URL to the DTD
*
* Load and parse an external subset.
*
* Returns the resulting xmlDtdPtr or NULL in case of error.
*/
#[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
pub fn xmlParseDTD(ExternalID: *const xmlChar, SystemID: *const xmlChar) -> xmlDtdPtr {
    return xmlSAXParseDTD(0 as xmlSAXHandlerPtr, ExternalID, SystemID);
}
/* LIBXML_VALID_ENABLED */
/* ***********************************************************************
*									*
*		Front ends when parsing an Entity			*
*									*
************************************************************************/
/* *
* xmlParseCtxtExternalEntity:
* @ctx:  the existing parsing context
* @URL:  the URL for the entity to load
* @ID:  the System ID for the entity to load
* @lst:  the return value for the set of parsed nodes
*
* Parse an external general entity within an existing parsing context
* An external general parsed entity is well-formed if it matches the
* production labeled extParsedEnt.
*
* [78] extParsedEnt ::= TextDecl? content
*
* Returns 0 if the entity is well formed, -1 in case of args problem and
*    the parser error code otherwise
*/

pub fn xmlParseCtxtExternalEntity(
    ctx: xmlParserCtxtPtr,
    URL: *const xmlChar,
    ID: *const xmlChar,
    lst: *mut xmlNodePtr,
) -> i32 {
    let mut userData: *mut () = 0 as *mut ();
    if ctx.is_null() {
        return -1;
    }
    /*
     * If the user provided their own SAX callbacks, then reuse the
     * userData callback field, otherwise the expected setup in a
     * DOM builder is to have userData == ctxt
     */
    let mut safe_ctx = unsafe { &mut *ctx };

    if safe_ctx.userData == ctx as *mut () {
        userData = 0 as *mut ()
    } else {
        userData = safe_ctx.userData
    }
    return unsafe {
        xmlParseExternalEntityPrivate(
            safe_ctx.myDoc,
            ctx,
            safe_ctx.sax,
            userData,
            safe_ctx.depth + 1,
            URL,
            ID,
            lst,
        ) as i32
    };
}

/* *
* xmlParseExternalEntityPrivate:
* @doc:  the document the chunk pertains to
* @oldctxt:  the previous parser context if available
* @sax:  the SAX handler block (possibly NULL)
* @user_data:  The user data returned on SAX callbacks (possibly NULL)
* @depth:  Used for loop detection, use 0
* @URL:  the URL for the entity to load
* @ID:  the System ID for the entity to load
* @list:  the return value for the set of parsed nodes
*
* Private version of xmlParseExternalEntity()
*
* Returns 0 if the entity is well formed, -1 in case of args problem and
*    the parser error code otherwise
*/
fn xmlParseExternalEntityPrivate(
    doc: xmlDocPtr,
    oldctxt: xmlParserCtxtPtr,
    sax: xmlSAXHandlerPtr,
    user_data: *mut (),
    depth: i32,
    URL: *const xmlChar,
    ID: *const xmlChar,
    list: *mut xmlNodePtr,
) -> xmlParserErrors {
    let mut ctxt: xmlParserCtxtPtr;
    let mut newDoc: xmlDocPtr;
    let mut newRoot: xmlNodePtr;
    let mut oldsax: xmlSAXHandlerPtr = 0 as xmlSAXHandlerPtr;
    let mut ret: xmlParserErrors = XML_ERR_OK;
    let mut start: [xmlChar; 4] = [0; 4];
    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
    let mut safe_oldctxt = unsafe { &mut *oldctxt };

    if depth > 40 && (oldctxt.is_null() || safe_oldctxt.options & XML_PARSE_HUGE as i32 == 0 as i32)
        || depth > 1024
    {
        return XML_ERR_ENTITY_LOOP;
    }
    unsafe {
        if !list.is_null() {
            *list = 0 as xmlNodePtr
        }
    }

    if URL.is_null() && ID.is_null() {
        return XML_ERR_INTERNAL_ERROR;
    }
    if doc.is_null() {
        return XML_ERR_INTERNAL_ERROR;
    }
    unsafe {
        ctxt = xmlCreateEntityParserCtxtInternal(URL, ID, 0 as *const xmlChar, oldctxt);
    }
    if ctxt.is_null() {
        return XML_WAR_UNDECLARED_ENTITY;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    safe_ctxt.userData = ctxt as *mut ();
    if !sax.is_null() {
        oldsax = safe_ctxt.sax;
        safe_ctxt.sax = sax;
        if !user_data.is_null() {
            safe_ctxt.userData = user_data
        }
    }
    unsafe {
        xmlDetectSAX2(ctxt);
        newDoc = xmlNewDoc(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar);
    }

    if newDoc.is_null() {
        unsafe {
            xmlFreeParserCtxt_safe(ctxt);
        }
        return XML_ERR_INTERNAL_ERROR;
    }
    let mut safe_newDoc = unsafe { &mut *newDoc };
    let mut safe_doc = unsafe { &mut *doc };
    safe_newDoc.properties = XML_DOC_INTERNAL as i32;
    if !doc.is_null() {
        safe_newDoc.intSubset = safe_doc.intSubset;
        safe_newDoc.extSubset = safe_doc.extSubset;
        if !safe_doc.dict.is_null() {
            safe_newDoc.dict = safe_doc.dict;
            unsafe {
                xmlDictReference(safe_newDoc.dict);
            }
        }
        if !safe_doc.URL.is_null() {
            safe_newDoc.URL = unsafe { xmlStrdup_safe(safe_doc.URL) }
        }
    }
    unsafe {
        newRoot = xmlNewDocNode(
            newDoc,
            0 as xmlNsPtr,
            b"pseudoroot\x00" as *const u8 as *const i8 as *mut xmlChar,
            0 as *const xmlChar,
        );
    }
    if newRoot.is_null() {
        if !sax.is_null() {
            safe_ctxt.sax = oldsax
        }
        unsafe { xmlFreeParserCtxt_safe(ctxt) };
        safe_newDoc.intSubset = 0 as *mut _xmlDtd;
        safe_newDoc.extSubset = 0 as *mut _xmlDtd;
        unsafe { xmlFreeDoc_safe(newDoc) };
        return XML_ERR_INTERNAL_ERROR;
    }
    unsafe {
        xmlAddChild(newDoc as xmlNodePtr, newRoot);
        nodePush(ctxt, safe_newDoc.children);
    }
    let mut safe_newRoot = unsafe { &mut *newRoot };
    if doc.is_null() {
        safe_ctxt.myDoc = newDoc
    } else {
        safe_ctxt.myDoc = doc;
        safe_newRoot.doc = doc
    }
    /*
     * Get the 4 first bytes and decode the charset
     * if enc != XML_CHAR_ENCODING_NONE
     * plug some encoding conversion routines.
     */
    unsafe {
        if safe_ctxt.progressive == 0
            && ((*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64) < 250
        {
            xmlGROW(ctxt);
        }
        if (*safe_ctxt.input).end.offset_from((*safe_ctxt.input).cur) as i64 >= 4 {
            start[0 as i32 as usize] = *(*safe_ctxt.input).cur;
            start[1 as i32 as usize] = *(*safe_ctxt.input).cur.offset(1);
            start[2 as i32 as usize] = *(*safe_ctxt.input).cur.offset(2);
            start[3 as i32 as usize] = *(*safe_ctxt.input).cur.offset(3);
            enc = xmlDetectCharEncoding(start.as_mut_ptr(), 4);
            if enc as i32 != XML_CHAR_ENCODING_NONE as i32 {
                xmlSwitchEncoding(ctxt, enc);
            }
        }
        /*
         * Parse a possible text declaration first
         */
        if *((*safe_ctxt.input).cur as *mut u8).offset(0) as i32 == '<' as i32
            && *((*safe_ctxt.input).cur as *mut u8).offset(1) as i32 == '?' as i32
            && *((*safe_ctxt.input).cur as *mut u8).offset(2) as i32 == 'x' as i32
            && *((*safe_ctxt.input).cur as *mut u8).offset(3) as i32 == 'm' as i32
            && *((*safe_ctxt.input).cur as *mut u8).offset(4) as i32 == 'l' as i32
            && (*(*safe_ctxt.input).cur.offset(5) as i32 == 0x20 as i32
                || 0x9 as i32 <= *(*safe_ctxt.input).cur.offset(5) as i32
                    && *(*safe_ctxt.input).cur.offset(5) as i32 <= 0xa as i32
                || *(*safe_ctxt.input).cur.offset(5) as i32 == 0xd as i32)
        {
            xmlParseTextDecl(ctxt);
            /*
             * An XML-1.0 document can't reference an entity not XML-1.0
             */
            if xmlStrEqual(
                safe_oldctxt.version,
                b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar,
            ) != 0
                && xmlStrEqual(
                    (*safe_ctxt.input).version,
                    b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar,
                ) == 0
            {
                xmlFatalErrMsg(
                    ctxt,
                    XML_ERR_VERSION_MISMATCH,
                    b"Version mismatch between document and entity\n\x00" as *const u8 as *const i8,
                );
            }
        }
    }
    safe_ctxt.instate = XML_PARSER_CONTENT;
    safe_ctxt.depth = depth;
    if !oldctxt.is_null() {
        safe_ctxt._private = safe_oldctxt._private;
        safe_ctxt.loadsubset = safe_oldctxt.loadsubset;
        safe_ctxt.validate = safe_oldctxt.validate;
        safe_ctxt.valid = safe_oldctxt.valid;
        safe_ctxt.replaceEntities = safe_oldctxt.replaceEntities;
        if safe_oldctxt.validate != 0 {
            safe_ctxt.vctxt.error = safe_oldctxt.vctxt.error;
            safe_ctxt.vctxt.warning = safe_oldctxt.vctxt.warning;
            safe_ctxt.vctxt.userData = safe_oldctxt.vctxt.userData
        }
        safe_ctxt.external = safe_oldctxt.external;
        if !safe_ctxt.dict.is_null() {
            unsafe {
                xmlDictFree_safe(safe_ctxt.dict);
            }
        }
        safe_ctxt.dict = safe_oldctxt.dict;
        unsafe {
            safe_ctxt.str_xml = xmlDictLookup(
                safe_ctxt.dict,
                b"xml\x00" as *const u8 as *const i8 as *mut xmlChar,
                3,
            );
            safe_ctxt.str_xmlns = xmlDictLookup(
                safe_ctxt.dict,
                b"xmlns\x00" as *const u8 as *const i8 as *mut xmlChar,
                5,
            );
            safe_ctxt.str_xml_ns = xmlDictLookup(
                safe_ctxt.dict,
                b"http://www.w3.org/XML/1998/namespace\x00" as *const u8 as *const i8
                    as *const xmlChar,
                36,
            );
        }
        safe_ctxt.dictNames = safe_oldctxt.dictNames;
        safe_ctxt.attsDefault = safe_oldctxt.attsDefault;
        safe_ctxt.attsSpecial = safe_oldctxt.attsSpecial;
        safe_ctxt.linenumbers = safe_oldctxt.linenumbers;
        safe_ctxt.record_info = safe_oldctxt.record_info;
        safe_ctxt.node_seq.maximum = safe_oldctxt.node_seq.maximum;
        safe_ctxt.node_seq.length = safe_oldctxt.node_seq.length;
        safe_ctxt.node_seq.buffer = safe_oldctxt.node_seq.buffer
    } else {
        /*
         * Doing validity checking on chunk without context
         * doesn't make sense
         */
        safe_ctxt._private = 0 as *mut ();
        safe_ctxt.validate = 0;
        safe_ctxt.external = 2;
        safe_ctxt.loadsubset = 0
    }
    unsafe {
        xmlParseContent(ctxt);
        if *(*safe_ctxt.input).cur as i32 == '<' as i32
            && *(*safe_ctxt.input).cur.offset(1) as i32 == '/' as i32
        {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        } else if *(*safe_ctxt.input).cur as i32 != 0 as i32 {
            xmlFatalErr(ctxt, XML_ERR_EXTRA_CONTENT, 0 as *const i8);
        }
        if safe_ctxt.node != safe_newDoc.children {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        }
    }
    if safe_ctxt.wellFormed == 0 {
        if safe_ctxt.errNo == 0 {
            ret = XML_ERR_INTERNAL_ERROR
        } else {
            ret = safe_ctxt.errNo as xmlParserErrors
        }
    } else {
        if !list.is_null() {
            let mut cur: xmlNodePtr = 0 as *mut xmlNode;
            /*
             * Return the newly created nodeset after unlinking it from
             * they pseudo parent.
             */
            unsafe {
                cur = (*safe_newDoc.children).children;
                *list = cur;
                while !cur.is_null() {
                    (*cur).parent = 0 as *mut _xmlNode;
                    cur = (*cur).next
                }
                (*safe_newDoc.children).children = 0 as *mut _xmlNode
            }
        }
        ret = XML_ERR_OK
    }
    /*
     * Record in the parent context the number of entities replacement
     * done when parsing that reference.
     */
    if !oldctxt.is_null() {
        safe_oldctxt.nbentities = safe_oldctxt.nbentities.wrapping_add(safe_ctxt.nbentities)
    }
    /*
     * Also record the size of the entity parsed
     */
    unsafe {
        if !safe_ctxt.input.is_null() && !oldctxt.is_null() {
            safe_oldctxt.sizeentities = safe_oldctxt
                .sizeentities
                .wrapping_add((*safe_ctxt.input).consumed);
            safe_oldctxt.sizeentities = safe_oldctxt.sizeentities.wrapping_add(
                (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 as u64,
            )
        }
        /*
         * And record the last error if any
         */
        if !oldctxt.is_null() && safe_ctxt.lastError.code != XML_ERR_OK as i32 {
            xmlCopyError(&mut safe_ctxt.lastError, &mut safe_oldctxt.lastError);
        }
    }
    if !sax.is_null() {
        safe_ctxt.sax = oldsax
    }
    if !oldctxt.is_null() {
        safe_ctxt.dict = 0 as xmlDictPtr;
        safe_ctxt.attsDefault = 0 as xmlHashTablePtr;
        safe_ctxt.attsSpecial = 0 as xmlHashTablePtr;
        safe_oldctxt.validate = safe_ctxt.validate;
        safe_oldctxt.valid = safe_ctxt.valid;
        safe_oldctxt.node_seq.maximum = safe_ctxt.node_seq.maximum;
        safe_oldctxt.node_seq.length = safe_ctxt.node_seq.length;
        safe_oldctxt.node_seq.buffer = safe_ctxt.node_seq.buffer
    }
    safe_ctxt.node_seq.maximum = 0;
    safe_ctxt.node_seq.length = 0;
    safe_ctxt.node_seq.buffer = 0 as *mut xmlParserNodeInfo;
    unsafe { xmlFreeParserCtxt_safe(ctxt) };
    safe_newDoc.intSubset = 0 as *mut _xmlDtd;
    safe_newDoc.extSubset = 0 as *mut _xmlDtd;
    unsafe { xmlFreeDoc_safe(newDoc) };
    return ret;
}
/* *
* xmlParseExternalEntity:
* @doc:  the document the chunk pertains to
* @sax:  the SAX handler block (possibly NULL)
* @user_data:  The user data returned on SAX callbacks (possibly NULL)
* @depth:  Used for loop detection, use 0
* @URL:  the URL for the entity to load
* @ID:  the System ID for the entity to load
* @lst:  the return value for the set of parsed nodes
*
* Parse an external general entity
* An external general parsed entity is well-formed if it matches the
* production labeled extParsedEnt.
*
* [78] extParsedEnt ::= TextDecl? content
*
* Returns 0 if the entity is well formed, -1 in case of args problem and
*    the parser error code otherwise
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseExternalEntity(
    doc: xmlDocPtr,
    sax: xmlSAXHandlerPtr,
    user_data: *mut (),
    depth: i32,
    URL: *const xmlChar,
    ID: *const xmlChar,
    lst: *mut xmlNodePtr,
) -> i32 {
    return xmlParseExternalEntityPrivate(
        doc,
        0 as xmlParserCtxtPtr,
        sax,
        user_data,
        depth,
        URL,
        ID,
        lst,
    ) as i32;
}
/* *
* xmlParseBalancedChunkMemory:
* @doc:  the document the chunk pertains to (must not be NULL)
* @sax:  the SAX handler block (possibly NULL)
* @user_data:  The user data returned on SAX callbacks (possibly NULL)
* @depth:  Used for loop detection, use 0
* @string:  the input string in UTF8 or ISO-Latin (zero terminated)
* @lst:  the return value for the set of parsed nodes
*
* Parse a well-balanced chunk of an XML document
* called by the parser
* The allowed sequence for the Well Balanced Chunk is the one defined by
* the content production in the XML grammar:
*
* [43] content ::= (element | CharData | Reference | CDSect | PI | Comment)*
*
* Returns 0 if the chunk is well balanced, -1 in case of args problem and
*    the parser error code otherwise
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseBalancedChunkMemory(
    doc: xmlDocPtr,
    sax: xmlSAXHandlerPtr,
    user_data: *mut (),
    depth: i32,
    string: *const xmlChar,
    lst: *mut xmlNodePtr,
) -> i32 {
    return xmlParseBalancedChunkMemoryRecover(doc, sax, user_data, depth, string, lst, 0 as i32);
}
/* LIBXML_LEGACY_ENABLED */
/* LIBXML_SAX1_ENABLED */
/* *
* xmlParseBalancedChunkMemoryInternal:
* @oldctxt:  the existing parsing context
* @string:  the input string in UTF8 or ISO-Latin (zero terminated)
* @user_data:  the user data field for the parser context
* @lst:  the return value for the set of parsed nodes
*
*
* Parse a well-balanced chunk of an XML document
* called by the parser
* The allowed sequence for the Well Balanced Chunk is the one defined by
* the content production in the XML grammar:
*
* [43] content ::= (element | CharData | Reference | CDSect | PI | Comment)*
*
* Returns XML_ERR_OK if the chunk is well balanced, and the parser
* error code otherwise
*
* In case recover is set to 1, the nodelist will not be empty even if
* the parsed chunk is not well balanced.
*/

fn xmlParseBalancedChunkMemoryInternal(
    oldctxt: xmlParserCtxtPtr,
    string: *const xmlChar,
    user_data: *mut (),
    lst: *mut xmlNodePtr,
) -> xmlParserErrors {
    let mut ctxt: xmlParserCtxtPtr;
    let mut newDoc: xmlDocPtr = 0 as xmlDocPtr;
    let mut newRoot: xmlNodePtr;
    let mut oldsax: xmlSAXHandlerPtr = 0 as xmlSAXHandlerPtr;
    let mut content: xmlNodePtr = 0 as xmlNodePtr;
    let mut last: xmlNodePtr = 0 as xmlNodePtr;
    let mut size: i32 = 0;
    let mut ret: xmlParserErrors = XML_ERR_OK;

    // match () {
    //     #[cfg(HAVE_parser_SAX2)] _ => {
    //         let mut i: i32 = 0;
    //     }
    //     #[cfg(not(HAVE_parser_SAX2))] _ => {}
    // };

    let mut safe_oldctxt = unsafe { &mut *oldctxt };
    if safe_oldctxt.depth > 40 && safe_oldctxt.options & XML_PARSE_HUGE as i32 == 0
        || safe_oldctxt.depth > 1024
    {
        return XML_ERR_ENTITY_LOOP;
    }
    unsafe {
        if !lst.is_null() {
            *lst = 0 as xmlNodePtr
        }
    }
    if string.is_null() {
        return XML_ERR_INTERNAL_ERROR;
    }
    unsafe {
        size = xmlStrlen_safe(string);
        ctxt = xmlCreateMemoryParserCtxt_safe(string as *mut i8, size);
    }
    if ctxt.is_null() {
        return XML_WAR_UNDECLARED_ENTITY;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if !user_data.is_null() {
        safe_ctxt.userData = user_data
    } else {
        safe_ctxt.userData = ctxt as *mut ()
    }
    unsafe {
        if !safe_ctxt.dict.is_null() {
            xmlDictFree(safe_ctxt.dict);
        }
    }
    safe_ctxt.dict = safe_oldctxt.dict;
    safe_ctxt.input_id = safe_oldctxt.input_id + 1;
    unsafe {
        safe_ctxt.str_xml = xmlDictLookup(
            safe_ctxt.dict,
            b"xml\x00" as *const u8 as *const i8 as *mut xmlChar,
            3,
        );
        safe_ctxt.str_xmlns = xmlDictLookup(
            safe_ctxt.dict,
            b"xmlns\x00" as *const u8 as *const i8 as *mut xmlChar,
            5,
        );
        safe_ctxt.str_xml_ns = xmlDictLookup(
            safe_ctxt.dict,
            b"http://www.w3.org/XML/1998/namespace\x00" as *const u8 as *const i8 as *const xmlChar,
            36,
        );
    }

    match () {
        #[cfg(HAVE_parser_SAX2)]
        _ => {
            unsafe {
                /* propagate namespaces down the entity */
                let mut i: i32 = 0;
                while i < safe_oldctxt.nsNr {
                    nsPush(
                        ctxt,
                        *safe_oldctxt.nsTab.offset(i as isize),
                        *safe_oldctxt.nsTab.offset((i + 1) as isize),
                    );
                    i += 2
                }
            }
        }
        #[cfg(not(HAVE_parser_SAX2))]
        _ => {}
    };

    oldsax = safe_ctxt.sax;
    safe_ctxt.sax = safe_oldctxt.sax;
    unsafe {
        xmlDetectSAX2(ctxt);
    }
    safe_ctxt.replaceEntities = safe_oldctxt.replaceEntities;
    safe_ctxt.options = safe_oldctxt.options;
    safe_ctxt._private = safe_oldctxt._private;
    if safe_oldctxt.myDoc.is_null() {
        unsafe {
            newDoc = xmlNewDoc(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar);
        }
        if newDoc.is_null() {
            safe_ctxt.sax = oldsax;
            safe_ctxt.dict = 0 as xmlDictPtr;
            unsafe {
                xmlFreeParserCtxt_safe(ctxt);
            }
            return XML_ERR_INTERNAL_ERROR;
        }
        let mut safe_newDoc = unsafe { &mut *newDoc };

        safe_newDoc.properties = XML_DOC_INTERNAL as i32;
        safe_newDoc.dict = safe_ctxt.dict;
        unsafe {
            xmlDictReference(safe_newDoc.dict);
        }
        safe_ctxt.myDoc = newDoc
    } else {
        safe_ctxt.myDoc = safe_oldctxt.myDoc;
        unsafe {
            content = (*safe_ctxt.myDoc).children;
            last = (*safe_ctxt.myDoc).last
        }
    }
    unsafe {
        newRoot = xmlNewDocNode(
            safe_ctxt.myDoc,
            0 as xmlNsPtr,
            b"pseudoroot\x00" as *const u8 as *const i8 as *mut xmlChar,
            0 as *const xmlChar,
        );
        if newRoot.is_null() {
            safe_ctxt.sax = oldsax;
            safe_ctxt.dict = 0 as xmlDictPtr;
            xmlFreeParserCtxt(ctxt);
            if !newDoc.is_null() {
                xmlFreeDoc(newDoc);
            }
            return XML_ERR_INTERNAL_ERROR;
        }
        (*safe_ctxt.myDoc).children = 0 as *mut _xmlNode;
        (*safe_ctxt.myDoc).last = 0 as *mut _xmlNode;
        xmlAddChild(safe_ctxt.myDoc as xmlNodePtr, newRoot);
        nodePush(ctxt, (*safe_ctxt.myDoc).children);
    }
    safe_ctxt.instate = XML_PARSER_CONTENT;
    safe_ctxt.depth = safe_oldctxt.depth + 1;
    safe_ctxt.validate = 0;
    safe_ctxt.loadsubset = safe_oldctxt.loadsubset;
    if safe_oldctxt.validate != 0 || safe_oldctxt.replaceEntities != 0 as i32 {
        /*
         * ID/IDREF registration will be done in xmlValidateElement below
         */
        safe_ctxt.loadsubset |= 8
    }
    safe_ctxt.dictNames = safe_oldctxt.dictNames;
    safe_ctxt.attsDefault = safe_oldctxt.attsDefault;
    safe_ctxt.attsSpecial = safe_oldctxt.attsSpecial;
    unsafe {
        xmlParseContent(ctxt);
        if *(*safe_ctxt.input).cur as i32 == '<' as i32
            && *(*safe_ctxt.input).cur.offset(1 as i32 as isize) as i32 == '/' as i32
        {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        } else if *(*safe_ctxt.input).cur as i32 != 0 as i32 {
            xmlFatalErr(ctxt, XML_ERR_EXTRA_CONTENT, 0 as *const i8);
        }
        if safe_ctxt.node != (*safe_ctxt.myDoc).children {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        }
    }
    if safe_ctxt.wellFormed == 0 {
        if safe_ctxt.errNo == 0 as i32 {
            ret = XML_ERR_INTERNAL_ERROR
        } else {
            ret = safe_ctxt.errNo as xmlParserErrors
        }
    } else {
        ret = XML_ERR_OK
    }
    if !lst.is_null() && ret as u32 == XML_ERR_OK as i32 as u32 {
        let mut cur: xmlNodePtr = 0 as *mut xmlNode;
        /*
         * Return the newly created nodeset after unlinking it from
         * they pseudo parent.
         */
        unsafe {
            cur = (*(*safe_ctxt.myDoc).children).children;
            *lst = cur;
        }
        while !cur.is_null() {
            match () {
                #[cfg(HAVE_parser_LIBXML_VALID_ENABLED)]
                _ => {
                    let mut safe_cur = unsafe { &mut *cur };

                    if safe_oldctxt.validate != 0
                        && safe_oldctxt.wellFormed != 0
                        && !safe_oldctxt.myDoc.is_null()
                        && unsafe { !(*safe_oldctxt.myDoc).intSubset.is_null() }
                        && safe_cur.type_0 as u32 == XML_ELEMENT_NODE as i32 as u32
                    {
                        unsafe {
                            safe_oldctxt.valid &=
                                xmlValidateElement(&mut safe_oldctxt.vctxt, safe_oldctxt.myDoc, cur)
                        }
                    }
                }
                #[cfg(not(HAVE_parser_LIBXML_VALID_ENABLED))]
                _ => {}
            };
            let mut safe_cur = unsafe { &mut *cur };
            safe_cur.parent = 0 as *mut _xmlNode;
            cur = safe_cur.next
        }
        unsafe { (*(*safe_ctxt.myDoc).children).children = 0 as *mut _xmlNode }
    }
    unsafe {
        if !safe_ctxt.myDoc.is_null() {
            xmlFreeNode((*safe_ctxt.myDoc).children);
            (*safe_ctxt.myDoc).children = content;
            (*safe_ctxt.myDoc).last = last
        }
    }
    /*
     * Record in the parent context the number of entities replacement
     * done when parsing that reference.
     */
    if !oldctxt.is_null() {
        safe_oldctxt.nbentities = safe_oldctxt.nbentities.wrapping_add(safe_ctxt.nbentities)
    }
    /*
     * Also record the last error if any
     */
    if safe_ctxt.lastError.code != XML_ERR_OK as i32 {
        unsafe {
            xmlCopyError(&mut safe_ctxt.lastError, &mut safe_oldctxt.lastError);
        }
    }
    safe_ctxt.sax = oldsax;
    safe_ctxt.dict = 0 as xmlDictPtr;
    safe_ctxt.attsDefault = 0 as xmlHashTablePtr;
    safe_ctxt.attsSpecial = 0 as xmlHashTablePtr;
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
        if !newDoc.is_null() {
            xmlFreeDoc_safe(newDoc);
        }
    }
    return ret;
}
/* *
* xmlParseInNodeContext:
* @node:  the context node
* @data:  the input string
* @datalen:  the input string length in bytes
* @options:  a combination of xmlParserOption
* @lst:  the return value for the set of parsed nodes
*
* Parse a well-balanced chunk of an XML document
* within the context (DTD, namespaces, etc ...) of the given node.
*
* The allowed sequence for the data is a Well Balanced Chunk defined by
* the content production in the XML grammar:
*
* [43] content ::= (element | CharData | Reference | CDSect | PI | Comment)*
*
* Returns XML_ERR_OK if the chunk is well balanced, and the parser
* error code otherwise
*/

pub fn xmlParseInNodeContext(
    mut node: xmlNodePtr,
    data: *const i8,
    datalen: i32,
    mut options: i32,
    lst: *mut xmlNodePtr,
) -> xmlParserErrors {
    match () {
        #[cfg(HAVE_parser_SAX2)]
        _ => {
            let mut ctxt: xmlParserCtxtPtr;
            let mut doc: xmlDocPtr = 0 as xmlDocPtr;
            let mut fake: xmlNodePtr;
            let mut cur: xmlNodePtr;
            let mut nsnr: i32 = 0 as i32;
            let mut ret: xmlParserErrors = XML_ERR_OK;
            /*
             * check all input parameters, grab the document
             */
            if lst.is_null() || node.is_null() || data.is_null() || datalen < 0 as i32 {
                return XML_ERR_INTERNAL_ERROR;
            }
            let mut safe_node = unsafe { &mut *node };

            match safe_node.type_0 as u32 {
                XML_ELEMENT_NODE
                | XML_ATTRIBUTE_NODE
                | XML_TEXT_NODE
                | XML_CDATA_SECTION_NODE
                | XML_ENTITY_REF_NODE
                | XML_PI_NODE
                | XML_COMMENT_NODE
                | XML_DOCUMENT_NODE
                | XML_HTML_DOCUMENT_NODE => {}
                _ => return XML_ERR_INTERNAL_ERROR,
            }
            while !node.is_null()
                && safe_node.type_0 as u32 != XML_ELEMENT_NODE as i32 as u32
                && safe_node.type_0 as u32 != XML_DOCUMENT_NODE as i32 as u32
                && safe_node.type_0 as u32 != XML_HTML_DOCUMENT_NODE as i32 as u32
            {
                node = safe_node.parent
            }
            if node.is_null() {
                return XML_ERR_INTERNAL_ERROR;
            }
            if safe_node.type_0 as u32 == XML_ELEMENT_NODE as i32 as u32 {
                doc = safe_node.doc
            } else {
                doc = node as xmlDocPtr
            }
            if doc.is_null() {
                return XML_ERR_INTERNAL_ERROR;
            }
            /*
             * allocate a context and set-up everything not related to the
             * node position in the tree
             */
            let mut safe_doc = unsafe { &mut *doc };

            if safe_doc.type_0 as u32 == XML_DOCUMENT_NODE as i32 as u32 {
                ctxt = unsafe { xmlCreateMemoryParserCtxt_safe(data as *mut i8, datalen) }
            } else if safe_doc.type_0 as u32 == XML_HTML_DOCUMENT_NODE as i32 as u32 {
                match () {
                    #[cfg(HAVE_parser_LIBXML_HTML_ENABLED)]
                    _ => {
                        unsafe {
                            ctxt = htmlCreateMemoryParserCtxt(data as *mut i8, datalen);
                        }
                        /*
                         * When parsing in context, it makes no sense to add implied
                         * elements like html/body/etc...
                         */
                        options |= HTML_PARSE_NOIMPLIED as i32;
                    }
                    #[cfg(not(HAVE_parser_LIBXML_HTML_ENABLED))]
                    _ => {}
                };
            } else {
                return XML_ERR_INTERNAL_ERROR;
            }
            if ctxt.is_null() {
                return XML_ERR_NO_MEMORY;
            }
            /*
             * Use input doc's dict if present, else assure XML_PARSE_NODICT is set.
             * We need a dictionary for xmlDetectSAX2, so if there's no doc dict
             * we must wait until the last moment to free the original one.
             */
            let mut safe_ctxt = unsafe { &mut *ctxt };

            if !safe_doc.dict.is_null() {
                if !safe_ctxt.dict.is_null() {
                    unsafe {
                        xmlDictFree(safe_ctxt.dict);
                    }
                }
                safe_ctxt.dict = safe_doc.dict
            } else {
                options |= XML_PARSE_NODICT as i32
            }
            if !safe_doc.encoding.is_null() {
                let mut hdlr: xmlCharEncodingHandlerPtr = 0 as *mut xmlCharEncodingHandler;
                if !safe_ctxt.encoding.is_null() {
                    unsafe {
                        xmlFree_safe(safe_ctxt.encoding as *mut xmlChar as *mut ());
                    }
                }
                unsafe {
                    safe_ctxt.encoding = xmlStrdup(safe_doc.encoding);
                    hdlr = xmlFindCharEncodingHandler(safe_doc.encoding as *const i8);
                    if !hdlr.is_null() {
                        xmlSwitchToEncoding(ctxt, hdlr);
                    } else {
                        return XML_ERR_UNSUPPORTED_ENCODING;
                    }
                }
            }
            xmlCtxtUseOptionsInternal(ctxt, options, 0 as *const i8);
            unsafe {
                xmlDetectSAX2(ctxt);
            }
            safe_ctxt.myDoc = doc;
            /* parsing in context, i.e. as within existing content */
            safe_ctxt.input_id = 2;
            safe_ctxt.instate = XML_PARSER_CONTENT;
            unsafe {
                fake = xmlNewComment(0 as *const xmlChar);
            }
            if fake.is_null() {
                unsafe {
                    xmlFreeParserCtxt_safe(ctxt);
                }
                return XML_ERR_NO_MEMORY;
            }
            unsafe {
                xmlAddChild(node, fake);
            }
            if safe_node.type_0 as u32 == XML_ELEMENT_NODE as i32 as u32 {
                unsafe {
                    nodePush(ctxt, node);
                }
                /*
                 * initialize the SAX2 namespaces stack
                 */
                cur = node;
                let mut safe_cur = unsafe { &mut *cur };
                while !cur.is_null() && safe_cur.type_0 as u32 == XML_ELEMENT_NODE as i32 as u32 {
                    let mut ns: xmlNsPtr = safe_cur.nsDef;
                    let mut iprefix: *const xmlChar = 0 as *const xmlChar;
                    let mut ihref: *const xmlChar = 0 as *const xmlChar;
                    while !ns.is_null() {
                        unsafe {
                            if !safe_ctxt.dict.is_null() {
                                iprefix = xmlDictLookup(safe_ctxt.dict, (*ns).prefix, -1);
                                ihref = xmlDictLookup(safe_ctxt.dict, (*ns).href, -1)
                            } else {
                                iprefix = (*ns).prefix;
                                ihref = (*ns).href
                            }
                        }
                        unsafe {
                            if xmlGetNamespace(ctxt, iprefix).is_null() {
                                nsPush(ctxt, iprefix, ihref);
                                nsnr += 1
                            }
                            ns = (*ns).next
                        }
                    }
                    cur = safe_cur.parent
                }
            }
            if safe_ctxt.validate != 0 || safe_ctxt.replaceEntities != 0 as i32 {
                /*
                 * ID/IDREF registration will be done in xmlValidateElement below
                 */
                safe_ctxt.loadsubset |= 8 as i32
            }

            match () {
                #[cfg(HAVE_parser_LIBXML_HTML_ENABLED)]
                _ => unsafe {
                    if safe_doc.type_0 as u32 == XML_HTML_DOCUMENT_NODE as i32 as u32 {
                        __htmlParseContent(ctxt as *mut ());
                    } else {
                        xmlParseContent(ctxt);
                    }
                },
                #[cfg(not(HAVE_parser_LIBXML_HTML_ENABLED))]
                _ => {
                    xmlParseContent(ctxt);
                }
            };
            unsafe {
                nsPop(ctxt, nsnr);
                if *(*safe_ctxt.input).cur as i32 == '<' as i32
                    && *(*safe_ctxt.input).cur.offset(1) as i32 == '/' as i32
                {
                    xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
                } else if *(*safe_ctxt.input).cur as i32 != 0 {
                    xmlFatalErr(ctxt, XML_ERR_EXTRA_CONTENT, 0 as *const i8);
                }
                if !safe_ctxt.node.is_null() && safe_ctxt.node != node {
                    xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
                    safe_ctxt.wellFormed = 0
                }
            }
            if safe_ctxt.wellFormed == 0 {
                if safe_ctxt.errNo == 0 {
                    ret = XML_ERR_INTERNAL_ERROR
                } else {
                    ret = safe_ctxt.errNo as xmlParserErrors
                }
            } else {
                ret = XML_ERR_OK
            }
            /*
             * Return the newly created nodeset after unlinking it from
             * the pseudo sibling.
             */
            let mut safe_fake = unsafe { &mut *fake };

            cur = safe_fake.next;
            safe_fake.next = 0 as *mut _xmlNode;
            safe_node.last = fake;
            let mut safe_cur = unsafe { &mut *cur };
            if !cur.is_null() {
                safe_cur.prev = 0 as *mut _xmlNode
            }
            unsafe {
                *lst = cur;
            }
            while !cur.is_null() {
                safe_cur.parent = 0 as *mut _xmlNode;
                cur = safe_cur.next
            }
            unsafe {
                xmlUnlinkNode(fake);
                xmlFreeNode(fake);
                if ret as u32 != XML_ERR_OK as i32 as u32 {
                    xmlFreeNodeList(*lst);

                    *lst = 0 as xmlNodePtr
                }
            }
            if !safe_doc.dict.is_null() {
                safe_ctxt.dict = 0 as xmlDictPtr
            }
            unsafe {
                xmlFreeParserCtxt_safe(ctxt);
            }
            return ret;
        }
        #[cfg(not(HAVE_parser_SAX2))]
        _ => {
            return XML_ERR_INTERNAL_ERROR;
        }
    };
    /* !SAX2 */
}
/* *
* xmlParseBalancedChunkMemoryRecover:
* @doc:  the document the chunk pertains to (must not be NULL)
* @sax:  the SAX handler block (possibly NULL)
* @user_data:  The user data returned on SAX callbacks (possibly NULL)
* @depth:  Used for loop detection, use 0
* @string:  the input string in UTF8 or ISO-Latin (zero terminated)
* @lst:  the return value for the set of parsed nodes
* @recover: return nodes even if the data is broken (use 0)
*
*
* Parse a well-balanced chunk of an XML document
* called by the parser
* The allowed sequence for the Well Balanced Chunk is the one defined by
* the content production in the XML grammar:
*
* [43] content ::= (element | CharData | Reference | CDSect | PI | Comment)*
*
* Returns 0 if the chunk is well balanced, -1 in case of args problem and
*    the parser error code otherwise
*
* In case recover is set to 1, the nodelist will not be empty even if
* the parsed chunk is not well balanced, assuming the parsing succeeded to
* some extent.
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseBalancedChunkMemoryRecover(
    doc: xmlDocPtr,
    sax: xmlSAXHandlerPtr,
    user_data: *mut (),
    depth: i32,
    string: *const xmlChar,
    lst: *mut xmlNodePtr,
    recover: i32,
) -> i32 {
    let mut ctxt: xmlParserCtxtPtr;
    let mut newDoc: xmlDocPtr;
    let mut oldsax: xmlSAXHandlerPtr = 0 as xmlSAXHandlerPtr;
    let mut content: xmlNodePtr;
    let mut newRoot: xmlNodePtr;
    let mut size: i32 = 0;
    let mut ret: i32 = 0;
    if depth > 40 {
        return XML_ERR_ENTITY_LOOP as i32;
    }
    unsafe {
        if !lst.is_null() {
            *lst = 0 as xmlNodePtr
        }
    }
    if string.is_null() {
        return -1;
    }
    unsafe {
        size = xmlStrlen_safe(string);
        ctxt = xmlCreateMemoryParserCtxt_safe(string as *mut i8, size);
    }
    if ctxt.is_null() {
        return -1;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    safe_ctxt.userData = ctxt as *mut ();
    if !sax.is_null() {
        oldsax = safe_ctxt.sax;
        safe_ctxt.sax = sax;
        if !user_data.is_null() {
            safe_ctxt.userData = user_data
        }
    }
    unsafe {
        newDoc = xmlNewDoc(b"1.0\x00" as *const u8 as *const i8 as *mut xmlChar);
    }
    unsafe {
        if newDoc.is_null() {
            xmlFreeParserCtxt(ctxt);
            return -1;
        }
    }
    let mut safe_newDoc = unsafe { &mut *newDoc };
    let mut safe_doc = unsafe { &mut *doc };

    safe_newDoc.properties = XML_DOC_INTERNAL as i32;
    if !doc.is_null() && !safe_doc.dict.is_null() {
        unsafe {
            xmlDictFree(safe_ctxt.dict);
            safe_ctxt.dict = safe_doc.dict;
            xmlDictReference(safe_ctxt.dict);
            safe_ctxt.str_xml = xmlDictLookup(
                safe_ctxt.dict,
                b"xml\x00" as *const u8 as *const i8 as *mut xmlChar,
                3,
            );
            safe_ctxt.str_xmlns = xmlDictLookup(
                safe_ctxt.dict,
                b"xmlns\x00" as *const u8 as *const i8 as *mut xmlChar,
                5,
            );
            safe_ctxt.str_xml_ns = xmlDictLookup(
                safe_ctxt.dict,
                b"http://www.w3.org/XML/1998/namespace\x00" as *const u8 as *const i8
                    as *const xmlChar,
                36,
            );
        }
        safe_ctxt.dictNames = 1
    } else {
        xmlCtxtUseOptionsInternal(ctxt, XML_PARSE_NODICT as i32, 0 as *const i8);
    }
    /* doc == NULL is only supported for historic reasons */
    if !doc.is_null() {
        safe_newDoc.intSubset = safe_doc.intSubset;
        safe_newDoc.extSubset = safe_doc.extSubset
    }
    unsafe {
        newRoot = xmlNewDocNode(
            newDoc,
            0 as xmlNsPtr,
            b"pseudoroot\x00" as *const u8 as *const i8 as *mut xmlChar,
            0 as *const xmlChar,
        );
    }

    if newRoot.is_null() {
        if !sax.is_null() {
            safe_ctxt.sax = oldsax
        }
        unsafe {
            xmlFreeParserCtxt_safe(ctxt);
        }
        safe_newDoc.intSubset = 0 as *mut _xmlDtd;
        safe_newDoc.extSubset = 0 as *mut _xmlDtd;
        unsafe {
            xmlFreeDoc_safe(newDoc);
        }
        return -1;
    }
    unsafe {
        xmlAddChild(newDoc as xmlNodePtr, newRoot);
        nodePush(ctxt, newRoot);
    }

    /* doc == NULL is only supported for historic reasons */
    if doc.is_null() {
        safe_ctxt.myDoc = newDoc
    } else {
        safe_ctxt.myDoc = newDoc;
        unsafe {
            (*safe_newDoc.children).doc = doc;
            /* Ensure that doc has XML spec namespace */
            xmlSearchNsByHref(
                doc,
                doc as xmlNodePtr,
                b"http://www.w3.org/XML/1998/namespace\x00" as *const u8 as *const i8
                    as *const xmlChar,
            );
        }
        safe_newDoc.oldNs = safe_doc.oldNs
    }
    safe_ctxt.instate = XML_PARSER_CONTENT;
    safe_ctxt.input_id = 2;
    safe_ctxt.depth = depth;
    /*
     * Doing validity checking on chunk doesn't make sense
     */
    safe_ctxt.validate = 0;
    safe_ctxt.loadsubset = 0;
    unsafe {
        xmlDetectSAX2(ctxt);
        let mut safe_doc = unsafe { &mut *doc };
        if !doc.is_null() {
            content = safe_doc.children;
            safe_doc.children = 0 as *mut _xmlNode;
            xmlParseContent(ctxt);
            safe_doc.children = content
        } else {
            xmlParseContent(ctxt);
        }
        if *(*safe_ctxt.input).cur as i32 == '<' as i32
            && *(*safe_ctxt.input).cur.offset(1 as i32 as isize) as i32 == '/' as i32
        {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        } else if *(*safe_ctxt.input).cur as i32 != 0 as i32 {
            xmlFatalErr(ctxt, XML_ERR_EXTRA_CONTENT, 0 as *const i8);
        }
        if safe_ctxt.node != safe_newDoc.children {
            xmlFatalErr(ctxt, XML_ERR_NOT_WELL_BALANCED, 0 as *const i8);
        }
    }
    if safe_ctxt.wellFormed == 0 {
        if safe_ctxt.errNo == 0 {
            ret = 1
        } else {
            ret = safe_ctxt.errNo
        }
    } else {
        ret = 0
    }
    if !lst.is_null() && (ret == 0 || recover == 1) {
        let mut cur: xmlNodePtr = 0 as *mut xmlNode;
        /*
         * Return the newly created nodeset after unlinking it from
         * they pseudo parent.
         */
        unsafe {
            cur = (*safe_newDoc.children).children;
            *lst = cur;
        }

        while !cur.is_null() {
            unsafe {
                xmlSetTreeDoc(cur, doc);
            }
            let mut safe_cur = unsafe { &mut *cur };
            safe_cur.parent = 0 as *mut _xmlNode;
            cur = safe_cur.next
        }
        unsafe { (*safe_newDoc.children).children = 0 as *mut _xmlNode }
    }
    if !sax.is_null() {
        safe_ctxt.sax = oldsax
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    safe_newDoc.intSubset = 0 as *mut _xmlDtd;
    safe_newDoc.extSubset = 0 as *mut _xmlDtd;
    /* This leaks the namespace list if doc == NULL */
    safe_newDoc.oldNs = 0 as *mut _xmlNs;
    unsafe {
        xmlFreeDoc_safe(newDoc);
    }
    return ret;
}
/* *
* xmlSAXParseEntity:
* @sax:  the SAX handler block
* @filename:  the filename
*
* parse an XML external entity out of context and build a tree.
* It use the given SAX function block to handle the parsing callback.
* If sax is NULL, fallback to the default DOM tree building routines.
*
* [78] extParsedEnt ::= TextDecl? content
*
* This correspond to a "Well Balanced" chunk
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseEntity(sax: xmlSAXHandlerPtr, filename: *const i8) -> xmlDocPtr {
    let mut ret: xmlDocPtr;
    let mut ctxt: xmlParserCtxtPtr;
    ctxt = xmlCreateFileParserCtxt(filename);
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if !sax.is_null() {
        if !safe_ctxt.sax.is_null() {
            unsafe {
                xmlFree_safe(safe_ctxt.sax as *mut ());
            }
        }
        safe_ctxt.sax = sax;
        safe_ctxt.userData = 0 as *mut ()
    }
    unsafe {
        xmlParseExtParsedEnt(ctxt);
    }
    if safe_ctxt.wellFormed != 0 {
        ret = safe_ctxt.myDoc
    } else {
        ret = 0 as xmlDocPtr;
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}

/* *
* xmlParseEntity:
* @filename:  the filename
*
* parse an XML external entity out of context and build a tree.
*
* [78] extParsedEnt ::= TextDecl? content
*
* This correspond to a "Well Balanced" chunk
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub unsafe fn xmlParseEntity(mut filename: *const i8) -> xmlDocPtr {
    return xmlSAXParseEntity(0 as xmlSAXHandlerPtr, filename);
}
/* LIBXML_SAX1_ENABLED */
/* *
* xmlCreateEntityParserCtxtInternal:
* @URL:  the entity URL
* @ID:  the entity PUBLIC ID
* @base:  a possible base for the target URI
* @pctx:  parser context used to set options on new context
*
* Create a parser context for an external entity
* Automatic support for ZLIB/Compress compressed document is provided
* by default if found at compile-time.
*
* Returns the new parser context or NULL
*/
fn xmlCreateEntityParserCtxtInternal(
    mut URL: *const xmlChar,
    ID: *const xmlChar,
    base: *const xmlChar,
    pctx: xmlParserCtxtPtr,
) -> xmlParserCtxtPtr {
    let mut ctxt: xmlParserCtxtPtr;
    let mut inputStream: xmlParserInputPtr;
    let mut directory: *mut i8 = 0 as *mut i8;
    let mut uri: *mut xmlChar;
    unsafe {
        ctxt = xmlNewParserCtxt_safe();
    }
    if ctxt.is_null() {
        return 0 as xmlParserCtxtPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_pctx = unsafe { &mut *pctx };

    if !pctx.is_null() {
        safe_ctxt.options = safe_pctx.options;
        safe_ctxt._private = safe_pctx._private;
        /*
         * this is a subparser of pctx, so the input_id should be
         * incremented to distinguish from main entity
         */
        safe_ctxt.input_id = safe_pctx.input_id + 1
    }
    /* Don't read from stdin. */
    unsafe {
        if xmlStrcmp(URL, b"-\x00" as *const u8 as *const i8 as *mut xmlChar) == 0 {
            URL = b"./-\x00" as *const u8 as *const i8 as *mut xmlChar
        }
    }
    unsafe {
        uri = xmlBuildURI(URL, base);
    }
    if uri.is_null() {
        unsafe {
            inputStream = xmlLoadExternalEntity_safe(URL as *mut i8, ID as *mut i8, ctxt);
        }
        if inputStream.is_null() {
            unsafe {
                xmlFreeParserCtxt_safe(ctxt);
            }
            return 0 as xmlParserCtxtPtr;
        }
        unsafe {
            inputPush_safe(ctxt, inputStream);
        }
        if safe_ctxt.directory.is_null() && directory.is_null() {
            unsafe { directory = xmlParserGetDirectory_safe(URL as *mut i8) }
        }
        if safe_ctxt.directory.is_null() && !directory.is_null() {
            safe_ctxt.directory = directory
        }
    } else {
        unsafe {
            inputStream = xmlLoadExternalEntity_safe(uri as *mut i8, ID as *mut i8, ctxt);
        }
        if inputStream.is_null() {
            unsafe {
                xmlFree_safe(uri as *mut ());
                xmlFreeParserCtxt_safe(ctxt);
            }
            return 0 as xmlParserCtxtPtr;
        }
        unsafe {
            inputPush_safe(ctxt, inputStream);
        }
        if safe_ctxt.directory.is_null() && directory.is_null() {
            unsafe { directory = xmlParserGetDirectory_safe(uri as *mut i8) }
        }
        if safe_ctxt.directory.is_null() && !directory.is_null() {
            safe_ctxt.directory = directory
        }
        unsafe {
            xmlFree_safe(uri as *mut ());
        }
    }
    return ctxt;
}

/* *
* xmlCreateEntityParserCtxt:
* @URL:  the entity URL
* @ID:  the entity PUBLIC ID
* @base:  a possible base for the target URI
*
* Create a parser context for an external entity
* Automatic support for ZLIB/Compress compressed document is provided
* by default if found at compile-time.
*
* Returns the new parser context or NULL
*/

pub fn xmlCreateEntityParserCtxt(
    URL: *const xmlChar,
    ID: *const xmlChar,
    base: *const xmlChar,
) -> xmlParserCtxtPtr {
    return xmlCreateEntityParserCtxtInternal(URL, ID, base, 0 as xmlParserCtxtPtr);
}
/* ***********************************************************************
*									*
*		Front ends when parsing from a file			*
*									*
************************************************************************/
/* *
* xmlCreateURLParserCtxt:
* @filename:  the filename or URL
* @options:  a combination of xmlParserOption
*
* Create a parser context for a file or URL content.
* Automatic support for ZLIB/Compress compressed document is provided
* by default if found at compile-time and for file accesses
*
* Returns the new parser context or NULL
*/

pub fn xmlCreateURLParserCtxt(filename: *const i8, options: i32) -> xmlParserCtxtPtr {
    let mut ctxt: xmlParserCtxtPtr;
    let mut inputStream: xmlParserInputPtr;
    let mut directory: *mut i8 = 0 as *mut i8;
    unsafe {
        ctxt = xmlNewParserCtxt_safe();
    }
    unsafe {
        if ctxt.is_null() {
            xmlErrMemory(
                0 as xmlParserCtxtPtr,
                b"cannot allocate parser context\x00" as *const u8 as *const i8,
            );
            return 0 as xmlParserCtxtPtr;
        }
    }
    if options != 0 {
        xmlCtxtUseOptionsInternal(ctxt, options, 0 as *const i8);
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    safe_ctxt.linenumbers = 1;
    unsafe {
        inputStream = xmlLoadExternalEntity_safe(filename, 0 as *const i8, ctxt);
    }
    if inputStream.is_null() {
        unsafe {
            xmlFreeParserCtxt_safe(ctxt);
        }
        return 0 as xmlParserCtxtPtr;
    }
    unsafe {
        inputPush_safe(ctxt, inputStream);
    }
    if safe_ctxt.directory.is_null() && directory.is_null() {
        unsafe { directory = xmlParserGetDirectory_safe(filename) }
    }
    if safe_ctxt.directory.is_null() && !directory.is_null() {
        safe_ctxt.directory = directory
    }
    return ctxt;
}
/* *
* xmlCreateFileParserCtxt:
* @filename:  the filename
*
* Create a parser context for a file content.
* Automatic support for ZLIB/Compress compressed document is provided
* by default if found at compile-time.
*
* Returns the new parser context or NULL
*/

pub fn xmlCreateFileParserCtxt(filename: *const i8) -> xmlParserCtxtPtr {
    return xmlCreateURLParserCtxt(filename, 0);
}
/* *
* xmlSAXParseFileWithData:
* @sax:  the SAX handler block
* @filename:  the filename
* @recovery:  work in recovery mode, i.e. tries to read no Well Formed
*             documents
* @data:  the userdata
*
* parse an XML file and build a tree. Automatic support for ZLIB/Compress
* compressed document is provided by default if found at compile-time.
* It use the given SAX function block to handle the parsing callback.
* If sax is NULL, fallback to the default DOM tree building routines.
*
* User data (void *) is stored within the parser context in the
* context's _private member, so it is available nearly everywhere in libxml
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseFileWithData(
    sax: xmlSAXHandlerPtr,
    filename: *const i8,
    recovery: i32,
    data: *mut (),
) -> xmlDocPtr {
    let mut ret: xmlDocPtr;
    let mut ctxt: xmlParserCtxtPtr;
    unsafe {
        xmlInitParser_safe();
    }
    ctxt = xmlCreateFileParserCtxt(filename);

    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !sax.is_null() {
        if !safe_ctxt.sax.is_null() {
            unsafe {
                xmlFree_safe(safe_ctxt.sax as *mut ());
            }
        }
        safe_ctxt.sax = sax
    }
    unsafe {
        xmlDetectSAX2(ctxt);
    }
    if !data.is_null() {
        safe_ctxt._private = data
    }
    if safe_ctxt.directory.is_null() {
        safe_ctxt.directory = unsafe { xmlParserGetDirectory_safe(filename) }
    }
    safe_ctxt.recovery = recovery;
    unsafe {
        xmlParseDocument(ctxt);
    }
    if safe_ctxt.wellFormed != 0 || recovery != 0 {
        ret = safe_ctxt.myDoc;
        unsafe {
            if !ret.is_null() && !(*safe_ctxt.input).buf.is_null() {
                if (*(*safe_ctxt.input).buf).compressed > 0 {
                    (*ret).compression = 9
                } else {
                    (*ret).compression = (*(*safe_ctxt.input).buf).compressed
                }
            }
        }
    } else {
        ret = 0 as xmlDocPtr;
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}

/* *
* xmlSAXParseFile:
* @sax:  the SAX handler block
* @filename:  the filename
* @recovery:  work in recovery mode, i.e. tries to read no Well Formed
*             documents
*
* parse an XML file and build a tree. Automatic support for ZLIB/Compress
* compressed document is provided by default if found at compile-time.
* It use the given SAX function block to handle the parsing callback.
* If sax is NULL, fallback to the default DOM tree building routines.
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseFile(sax: xmlSAXHandlerPtr, filename: *const i8, recovery: i32) -> xmlDocPtr {
    return xmlSAXParseFileWithData(sax, filename, recovery, 0 as *mut ());
}

/* *
* xmlRecoverDoc:
* @cur:  a pointer to an array of xmlChar
*
* parse an XML in-memory document and build a tree.
* In the case the document is not Well Formed, a attempt to build a
* tree is tried anyway
*
* Returns the resulting document tree or NULL in case of failure
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlRecoverDoc(mut cur: *const xmlChar) -> xmlDocPtr {
    return xmlSAXParseDoc(0 as xmlSAXHandlerPtr, cur, 1);
}
/* *
* xmlParseFile:
* @filename:  the filename
*
* parse an XML file and build a tree. Automatic support for ZLIB/Compress
* compressed document is provided by default if found at compile-time.
*
* Returns the resulting document tree if the file was wellformed,
* NULL otherwise.
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseFile(mut filename: *const i8) -> xmlDocPtr {
    return xmlSAXParseFile(0 as xmlSAXHandlerPtr, filename, 0);
}
/* *
* xmlRecoverFile:
* @filename:  the filename
*
* parse an XML file and build a tree. Automatic support for ZLIB/Compress
* compressed document is provided by default if found at compile-time.
* In the case the document is not Well Formed, it attempts to build
* a tree anyway
*
* Returns the resulting document tree or NULL in case of failure
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlRecoverFile(mut filename: *const i8) -> xmlDocPtr {
    return xmlSAXParseFile(0 as xmlSAXHandlerPtr, filename, 1);
}
/* *
* xmlSetupParserForBuffer:
* @ctxt:  an XML parser context
* @buffer:  a xmlChar * buffer
* @filename:  a file name
*
* Setup the parser context to parse a new buffer; Clears any prior
* contents from the parser context. The buffer parameter must not be
* NULL, but the filename parameter can be
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSetupParserForBuffer(
    ctxt: xmlParserCtxtPtr,
    buffer: *const xmlChar,
    filename: *const i8,
) {
    let mut input: xmlParserInputPtr = 0 as *mut xmlParserInput;
    if ctxt.is_null() || buffer.is_null() {
        return;
    }
    input = unsafe { xmlNewInputStream_safe(ctxt) };
    if input.is_null() {
        unsafe {
            xmlErrMemory(
                0 as xmlParserCtxtPtr,
                b"parsing new buffer: out of memory\n\x00" as *const u8 as *const i8,
            );
        }
        unsafe { xmlClearParserCtxt(ctxt) };
        return;
    }
    unsafe { xmlClearParserCtxt(ctxt) };
    let mut safe_input = unsafe { *input };
    if !filename.is_null() {
        unsafe { safe_input.filename = xmlCanonicPath_safe(filename as *const xmlChar) as *mut i8 }
    }
    safe_input.base = buffer;
    safe_input.cur = buffer;
    unsafe {
        safe_input.end = &*buffer
            .offset((xmlStrlen as unsafe extern "C" fn(_: *const xmlChar) -> i32)(buffer) as isize)
            as *const xmlChar;
    }
    unsafe { inputPush_safe(ctxt, input) };
}
/* *
* xmlSAXUserParseFile:
* @sax:  a SAX handler
* @user_data:  The user data returned on SAX callbacks
* @filename:  a file name
*
* parse an XML file and call the given SAX handler routines.
* Automatic support for ZLIB/Compress compressed document is provided
*
* Returns 0 in case of success or a error number otherwise
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXUserParseFile(sax: xmlSAXHandlerPtr, user_data: *mut (), filename: *const i8) -> i32 {
    let mut ret: i32 = 0;
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    unsafe {
        ctxt = xmlCreateFileParserCtxt(filename);
    }
    if ctxt.is_null() {
        return -1;
    }

    let mut safe_ctxt = unsafe { &mut *ctxt };
    unsafe {
        if safe_ctxt.sax != __xmlDefaultSAXHandler_safe() as xmlSAXHandlerPtr {
            xmlFree_safe(safe_ctxt.sax as *mut ());
        }
    }
    safe_ctxt.sax = sax;
    unsafe {
        xmlDetectSAX2(ctxt);
    }
    if !user_data.is_null() {
        safe_ctxt.userData = user_data
    }
    unsafe {
        xmlParseDocument(ctxt);
    }
    if safe_ctxt.wellFormed != 0 {
        ret = 0
    } else if safe_ctxt.errNo != 0 {
        ret = safe_ctxt.errNo
    } else {
        ret = -1
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    if !safe_ctxt.myDoc.is_null() {
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}
/* LIBXML_SAX1_ENABLED */
/* ***********************************************************************
*									*
*		Front ends when parsing from memory			*
*									*
************************************************************************/
/* *
* xmlCreateMemoryParserCtxt:
* @buffer:  a pointer to a char array
* @size:  the size of the array
*
* Create a parser context for an XML in-memory document.
*
* Returns the new parser context or NULL
*/

pub fn xmlCreateMemoryParserCtxt_parser(buffer: *const i8, size: i32) -> xmlParserCtxtPtr {
    let mut ctxt: xmlParserCtxtPtr;
    let mut input: xmlParserInputPtr;
    let mut buf: xmlParserInputBufferPtr;
    if buffer.is_null() {
        return 0 as xmlParserCtxtPtr;
    }
    if size <= 0 {
        return 0 as xmlParserCtxtPtr;
    }
    unsafe { ctxt = xmlNewParserCtxt_safe() };
    if ctxt.is_null() {
        return 0 as xmlParserCtxtPtr;
    }
    /* TODO: xmlParserInputBufferCreateStatic, requires some serious changes */
    unsafe {
        buf = xmlParserInputBufferCreateMem_safe(buffer, size, XML_CHAR_ENCODING_NONE);
    }
    if buf.is_null() {
        unsafe {
            xmlFreeParserCtxt_safe(ctxt);
        }
        return 0 as xmlParserCtxtPtr;
    }
    unsafe {
        input = xmlNewInputStream_safe(ctxt);
    }
    if input.is_null() {
        unsafe {
            xmlFreeParserInputBuffer_safe(buf);
            xmlFreeParserCtxt_safe(ctxt);
        }
        return 0 as xmlParserCtxtPtr;
    }
    unsafe {
        (*input).filename = 0 as *const i8;
        (*input).buf = buf;
        xmlBufResetInput((*(*input).buf).buffer, input);
        inputPush_safe(ctxt, input);
    }
    return ctxt;
}

/* *
* xmlSAXParseMemoryWithData:
* @sax:  the SAX handler block
* @buffer:  an pointer to a char array
* @size:  the size of the array
* @recovery:  work in recovery mode, i.e. tries to read no Well Formed
*             documents
* @data:  the userdata
*
* parse an XML in-memory block and use the given SAX function block
* to handle the parsing callback. If sax is NULL, fallback to the default
* DOM tree building routines.
*
* User data (void *) is stored within the parser context in the
* context's _private member, so it is available nearly everywhere in libxml
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseMemoryWithData(
    sax: xmlSAXHandlerPtr,
    buffer: *const i8,
    size: i32,
    recovery: i32,
    data: *mut (),
) -> xmlDocPtr {
    let ret: xmlDocPtr;
    let ctxt: xmlParserCtxtPtr;
    unsafe {
        xmlInitParser_safe();
        ctxt = xmlCreateMemoryParserCtxt_safe(buffer, size);
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !sax.is_null() {
        if !safe_ctxt.sax.is_null() {
            unsafe {
                xmlFree_safe(safe_ctxt.sax as *mut ());
            }
        }
        safe_ctxt.sax = sax
    }
    unsafe {
        xmlDetectSAX2(ctxt);
    }
    if !data.is_null() {
        safe_ctxt._private = data
    }
    safe_ctxt.recovery = recovery;
    unsafe {
        xmlParseDocument(ctxt);
    }
    if safe_ctxt.wellFormed != 0 || recovery != 0 {
        ret = safe_ctxt.myDoc
    } else {
        ret = 0 as xmlDocPtr;
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if !sax.is_null() {
        unsafe { safe_ctxt.sax = 0 as *mut _xmlSAXHandler }
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}

/* *
* xmlSAXParseMemory:
* @sax:  the SAX handler block
* @buffer:  an pointer to a char array
* @size:  the size of the array
* @recovery:  work in recovery mode, i.e. tries to read not Well Formed
*             documents
*
* parse an XML in-memory block and use the given SAX function block
* to handle the parsing callback. If sax is NULL, fallback to the default
* DOM tree building routines.
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseMemory(
    sax: xmlSAXHandlerPtr,
    buffer: *const i8,
    size: i32,
    recovery: i32,
) -> xmlDocPtr {
    return xmlSAXParseMemoryWithData(sax, buffer, size, recovery, 0 as *mut ());
}
/* *
* xmlParseMemory:
* @buffer:  an pointer to a char array
* @size:  the size of the array
*
* parse an XML in-memory block and build a tree.
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseMemory(mut buffer: *const i8, mut size: i32) -> xmlDocPtr {
    return xmlSAXParseMemory(0 as xmlSAXHandlerPtr, buffer, size, 0);
}
/* *
* xmlRecoverMemory:
* @buffer:  an pointer to a char array
* @size:  the size of the array
*
* parse an XML in-memory block and build a tree.
* In the case the document is not Well Formed, an attempt to
* build a tree is tried anyway
*
* Returns the resulting document tree or NULL in case of error
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub unsafe fn xmlRecoverMemory(mut buffer: *const i8, mut size: i32) -> xmlDocPtr {
    return xmlSAXParseMemory(0 as xmlSAXHandlerPtr, buffer, size, 1 as i32);
}
/* *
* xmlSAXUserParseMemory:
* @sax:  a SAX handler
* @user_data:  The user data returned on SAX callbacks
* @buffer:  an in-memory XML document input
* @size:  the length of the XML document in bytes
*
* A better SAX parsing routine.
* parse an XML in-memory buffer and call the given SAX handler routines.
*
* Returns 0 in case of success or a error number otherwise
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXUserParseMemory(
    sax: xmlSAXHandlerPtr,
    user_data: *mut (),
    buffer: *const i8,
    size: i32,
) -> i32 {
    let mut ret: i32 = 0 as i32;
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    unsafe {
        xmlInitParser_safe();
        ctxt = xmlCreateMemoryParserCtxt_safe(buffer, size);
    }
    if ctxt.is_null() {
        return -1;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    unsafe {
        if safe_ctxt.sax != __xmlDefaultSAXHandler_safe() as xmlSAXHandlerPtr {
            xmlFree_safe(safe_ctxt.sax as *mut ());
        }
    }
    safe_ctxt.sax = sax;
    unsafe {
        xmlDetectSAX2(ctxt);
    }
    if !user_data.is_null() {
        safe_ctxt.userData = user_data
    }
    unsafe {
        xmlParseDocument(ctxt);
    }
    if safe_ctxt.wellFormed != 0 {
        ret = 0
    } else if safe_ctxt.errNo != 0 as i32 {
        ret = safe_ctxt.errNo
    } else {
        ret = -1
    }
    if !sax.is_null() {
        safe_ctxt.sax = 0 as *mut _xmlSAXHandler
    }
    unsafe {
        if !safe_ctxt.myDoc.is_null() {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
            safe_ctxt.myDoc = 0 as xmlDocPtr
        }
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}
/* LIBXML_SAX1_ENABLED */
/* *
* xmlCreateDocParserCtxt:
* @cur:  a pointer to an array of xmlChar
*
* Creates a parser context for an XML in-memory document.
*
* Returns the new parser context or NULL
*/

pub fn xmlCreateDocParserCtxt(cur: *const xmlChar) -> xmlParserCtxtPtr {
    let mut len: i32 = 0;
    if cur.is_null() {
        return 0 as xmlParserCtxtPtr;
    }
    unsafe {
        len = xmlStrlen_safe(cur);
        return xmlCreateMemoryParserCtxt(cur as *const i8, len);
    }
}
/* *
* xmlSAXParseDoc:
* @sax:  the SAX handler block
* @cur:  a pointer to an array of xmlChar
* @recovery:  work in recovery mode, i.e. tries to read no Well Formed
*             documents
*
* parse an XML in-memory document and build a tree.
* It use the given SAX function block to handle the parsing callback.
* If sax is NULL, fallback to the default DOM tree building routines.
*
* Returns the resulting document tree
*/

#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlSAXParseDoc(sax: xmlSAXHandlerPtr, cur: *const xmlChar, recovery: i32) -> xmlDocPtr {
    let mut ret: xmlDocPtr;
    let mut ctxt: xmlParserCtxtPtr;
    let mut oldsax: xmlSAXHandlerPtr = 0 as xmlSAXHandlerPtr;
    if cur.is_null() {
        return 0 as xmlDocPtr;
    }
    ctxt = xmlCreateDocParserCtxt(cur);
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };

    if !sax.is_null() {
        oldsax = safe_ctxt.sax;
        safe_ctxt.sax = sax;
        safe_ctxt.userData = 0 as *mut ()
    }
    unsafe {
        xmlDetectSAX2(ctxt);
        xmlParseDocument(ctxt);
    }
    if safe_ctxt.wellFormed != 0 || recovery != 0 {
        ret = safe_ctxt.myDoc
    } else {
        ret = 0 as xmlDocPtr;
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
        safe_ctxt.myDoc = 0 as xmlDocPtr
    }
    if !sax.is_null() {
        safe_ctxt.sax = oldsax
    }
    unsafe {
        xmlFreeParserCtxt_safe(ctxt);
    }
    return ret;
}

/* *
* xmlParseDoc:
* @cur:  a pointer to an array of xmlChar
*
* parse an XML in-memory document and build a tree.
*
* Returns the resulting document tree
*/
#[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
pub fn xmlParseDoc(cur: *const xmlChar) -> xmlDocPtr {
    return xmlSAXParseDoc(0 as xmlSAXHandlerPtr, cur, 0 as i32);
}
/* LIBXML_SAX1_ENABLED */
/* ***********************************************************************
*									*
*	Specific function to keep track of entities references		*
*	and used by the XSLT debugger					*
*									*
************************************************************************/
#[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
static mut xmlEntityRefFunc: xmlEntityReferenceFunc = None;
/* *
* xmlAddEntityReference:
* @ent : A valid entity
* @firstNode : A valid first node for children of entity
* @lastNode : A valid last node of children entity
*
* Notify of a reference to an entity of type XML_EXTERNAL_GENERAL_PARSED_ENTITY
*/

#[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
fn xmlAddEntityReference(ent: xmlEntityPtr, firstNode: xmlNodePtr, lastNode: xmlNodePtr) {
    unsafe {
        if xmlEntityRefFunc.is_some() {
            Some(xmlEntityRefFunc.expect("non-null function pointer"))
                .expect("non-null function pointer")(ent, firstNode, lastNode);
        };
    }
}
/* *
* xmlSetEntityReferenceFunc:
* @func: A valid function
*
* Set the function to call call back when a xml reference has been made
*/
#[cfg(HAVE_parser_LIBXML_LEGACY_ENABLED)]
pub fn xmlSetEntityReferenceFunc(mut func: xmlEntityReferenceFunc) {
    unsafe {
        xmlEntityRefFunc = func;
    }
}
static mut xmlParserInitialized: i32 = 0 as i32;
/* *
* xmlInitParser:
*
* Initialization function for the XML parser.
* This is not reentrant. Call once before processing in case of
* use in multithreaded programs.
*/

pub unsafe fn xmlInitParser_parser() {
    unsafe {
        if xmlParserInitialized != 0 as i32 {
            return;
        }
    }

    // if cfg!(HAVE_parser_WIN32) && cfg!(not(HAVE_parser_LIBXML_STATIC)) || cfg!(HAVE_parser_LIBXML_STATIC_FOR_DLL) {
    //     //#if defined(_WIN32) && (!defined(LIBXML_STATIC) || defined(LIBXML_STATIC_FOR_DLL))
    //     atexit(Some(xmlCleanupParser as
    //         unsafe extern "C" fn() -> ()));
    //     //#endif
    // }

    match () {
        #[cfg(HAVE_parser_WIN32)]
        _ => {
            match () {
                #[cfg(HAVE_parser_LIBXML_STATIC)]
                _ => {
                    match () {
                        #[cfg(HAVE_parser_LIBXML_STATIC_FOR_DLL)]
                        _ => {
                            atexit(Some(xmlCleanupParser as unsafe extern "C" fn() -> ()));
                        }
                        #[cfg(not(HAVE_parser_LIBXML_STATIC_FOR_DLL))]
                        _ => {}
                    };
                }
                #[cfg(not(HAVE_parser_LIBXML_STATIC))]
                _ => {
                    atexit(Some(xmlCleanupParser as unsafe extern "C" fn() -> ()));
                }
            };
        }
        #[cfg(not(HAVE_parser_WIN32))]
        _ => {
            match () {
                #[cfg(HAVE_parser_LIBXML_STATIC_FOR_DLL)]
                _ => {
                    atexit(Some(xmlCleanupParser as unsafe extern "C" fn() -> ()));
                }
                #[cfg(not(HAVE_parser_LIBXML_STATIC_FOR_DLL))]
                _ => {}
            };
        }
    };

    match () {
        #[cfg(HAVE_parser_LIBXML_THREAD_ENABLED)]
        _ => {
            __xmlGlobalInitMutexLock_safe();
            if unsafe { xmlParserInitialized == 0 as i32 } {
                xmlInitThreads_safe();
                xmlInitGlobals_safe();
                unsafe {
                    if *__xmlGenericError()
                        == Some(
                            xmlGenericErrorDefaultFunc
                                as unsafe extern "C" fn(_: *mut (), _: *const i8, _: ...) -> (),
                        )
                        || (*__xmlGenericError()).is_none()
                    {
                        initGenericErrorDefaultFunc(0 as *mut xmlGenericErrorFunc);
                    }
                }
                xmlInitMemory_safe();
                xmlInitializeDict_safe();
                xmlInitCharEncodingHandlers_safe();
                xmlDefaultSAXHandlerInit_safe();
                xmlRegisterDefaultInputCallbacks_safe();
                match () {
                    #[cfg(HAVE_parser_LIBXML_OUTPUT_ENABLED)]
                    _ => {
                        xmlRegisterDefaultOutputCallbacks_safe();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_OUTPUT_ENABLED))]
                    _ => {}
                };
                /* LIBXML_OUTPUT_ENABLED */
                match () {
                    #[cfg(HAVE_parser_LIBXML_HTML_ENABLED)]
                    _ => {
                        htmlInitAutoClose_safe();
                        htmlDefaultSAXHandlerInit_safe();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_HTML_ENABLED))]
                    _ => {}
                };
                match () {
                    #[cfg(HAVE_parser_LIBXML_XPATH_ENABLED)]
                    _ => {
                        xmlXPathInit_safe();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_XPATH_ENABLED))]
                    _ => {}
                };
                unsafe { xmlParserInitialized = 1 as i32 }
            }
            __xmlGlobalInitMutexUnlock_safe();
        }

        #[cfg(not(HAVE_parser_LIBXML_THREAD_ENABLED))]
        _ => {
            unsafe {
                xmlInitThreads();
                xmlInitGlobals();
                if *__xmlGenericError()
                    == Some(
                        xmlGenericErrorDefaultFunc
                            as unsafe extern "C" fn(_: *mut (), _: *const i8, _: ...) -> (),
                    )
                    || (*__xmlGenericError()).is_none()
                {
                    initGenericErrorDefaultFunc(0 as *mut xmlGenericErrorFunc);
                }
                xmlInitMemory();
                xmlInitializeDict();
                xmlInitCharEncodingHandlers();
                xmlDefaultSAXHandlerInit();
                xmlRegisterDefaultInputCallbacks();
                match () {
                    #[cfg(HAVE_parser_LIBXML_OUTPUT_ENABLED)]
                    _ => {
                        xmlRegisterDefaultOutputCallbacks();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_OUTPUT_ENABLED))]
                    _ => {}
                };
                /* LIBXML_OUTPUT_ENABLED */
                match () {
                    #[cfg(HAVE_parser_LIBXML_HTML_ENABLED)]
                    _ => {
                        htmlInitAutoClose();
                        htmlDefaultSAXHandlerInit();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_HTML_ENABLED))]
                    _ => {}
                };
                match () {
                    #[cfg(HAVE_parser_LIBXML_XPATH_ENABLED)]
                    _ => {
                        xmlXPathInit();
                    }
                    #[cfg(not(HAVE_parser_LIBXML_XPATH_ENABLED))]
                    _ => {}
                };
                xmlParserInitialized = 1 as i32;
            }
        }
    };
}
/* *
* xmlCleanupParser:
*
* This function name is somewhat misleading. It does not clean up
* parser state, it cleans up memory allocated by the library itself.
* It is a cleanup function for the XML library. It tries to reclaim all
* related global memory allocated for the library processing.
* It doesn't deallocate any document related memory. One should
* call xmlCleanupParser() only when the process has finished using
* the library and all XML/HTML documents built with it.
* See also xmlInitParser() which has the opposite function of preparing
* the library for operations.
*
* WARNING: if your application is multithreaded or has plugin support
*          calling this may crash the application if another thread or
*          a plugin is still using libxml2. It's sometimes very hard to
*          guess if libxml2 is in use in the application, some libraries
*          or plugins may use it without notice. In case of doubt abstain
*          from calling this function or do it just before calling exit()
*          to avoid leak reports from valgrind !
*/

pub fn xmlCleanupParser() {
    unsafe {
        if xmlParserInitialized == 0 {
            return;
        }
    } /* must be last if called not from the main thread */
    unsafe {
        xmlCleanupCharEncodingHandlers_safe();
        match () {
            #[cfg(HAVE_parser_LIBXML_CATALOG_ENABLED)]
            _ => {
                xmlCatalogCleanup_safe();
            }
            #[cfg(not(HAVE_parser_LIBXML_CATALOG_ENABLED))]
            _ => {}
        };
        xmlDictCleanup_safe();
        xmlCleanupInputCallbacks_safe();
    }
    match () {
        #[cfg(HAVE_parser_LIBXML_OUTPUT_ENABLED)]
        _ => unsafe {
            xmlCleanupOutputCallbacks_safe();
        },
        #[cfg(not(HAVE_parser_LIBXML_OUTPUT_ENABLED))]
        _ => {}
    };
    match () {
        #[cfg(HAVE_parser_LIBXML_SCHEMAS_ENABLED)]
        _ => unsafe {
            xmlSchemaCleanupTypes_safe();
            xmlRelaxNGCleanupTypes_safe();
        },
        #[cfg(not(HAVE_parser_LIBXML_SCHEMAS_ENABLED))]
        _ => {}
    };
    unsafe {
        xmlResetLastError_safe();
        xmlCleanupGlobals_safe();
        xmlCleanupThreads_safe();
        xmlCleanupMemory_safe();
    }
    unsafe {
        xmlParserInitialized = 0;
    }
}
/* ***********************************************************************
*									*
*	New set (2.6.0) of simpler and more flexible APIs		*
*									*
************************************************************************/
/* *
* DICT_FREE:
* @str:  a string
*
* Free a string if it is not owned by the "dict" dictionary in the
* current scope
*/
/* *
* xmlCtxtReset:
* @ctxt: an XML parser context
*
* Reset a parser context
*/

pub fn xmlCtxtReset_parser(mut ctxt: xmlParserCtxtPtr) {
    let mut input: xmlParserInputPtr = 0 as *mut xmlParserInput;
    let mut dict: xmlDictPtr = 0 as *mut xmlDict;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if ctxt.is_null() {
        return;
    }
    dict = safe_ctxt.dict;
    loop {
        unsafe {
            input = inputPop_safe(ctxt);
        }
        if input.is_null() {
            break;
        }
        /* Non consuming */
        unsafe {
            xmlFreeInputStream_safe(input);
        }
    }
    safe_ctxt.inputNr = 0;
    safe_ctxt.input = 0 as xmlParserInputPtr;
    safe_ctxt.spaceNr = 0;
    unsafe {
        if !safe_ctxt.spaceTab.is_null() {
            *safe_ctxt.spaceTab.offset(0 as i32 as isize) = -(1 as i32);
            safe_ctxt.space = &mut *safe_ctxt.spaceTab.offset(0 as i32 as isize) as *mut i32
        } else {
            safe_ctxt.space = 0 as *mut i32
        }
    }
    safe_ctxt.nodeNr = 0;
    safe_ctxt.node = 0 as xmlNodePtr;
    safe_ctxt.nameNr = 0;
    safe_ctxt.name = 0 as *const xmlChar;
    if !safe_ctxt.version.is_null()
        && (dict.is_null() || unsafe { xmlDictOwns_safe(dict, safe_ctxt.version) } == 0)
    {
        unsafe {
            xmlFree_safe(safe_ctxt.version as *mut i8 as *mut ());
        }
    }
    safe_ctxt.version = 0 as *const xmlChar;
    if !safe_ctxt.encoding.is_null()
        && (dict.is_null() || unsafe { xmlDictOwns_safe(dict, safe_ctxt.encoding) } == 0 as i32)
    {
        unsafe {
            xmlFree_safe(safe_ctxt.encoding as *mut i8 as *mut ());
        }
    }
    safe_ctxt.encoding = 0 as *const xmlChar;
    if !safe_ctxt.directory.is_null()
        && (dict.is_null()
            || unsafe { xmlDictOwns_safe(dict, safe_ctxt.directory as *const xmlChar) } == 0)
    {
        unsafe {
            xmlFree_safe(safe_ctxt.directory as *mut ());
        }
    }
    safe_ctxt.directory = 0 as *mut i8;
    if !safe_ctxt.extSubURI.is_null()
        && (dict.is_null()
            || unsafe { xmlDictOwns_safe(dict, safe_ctxt.extSubURI as *const xmlChar) } == 0)
    {
        unsafe {
            xmlFree_safe(safe_ctxt.extSubURI as *mut i8 as *mut ());
        }
    }
    safe_ctxt.extSubURI = 0 as *mut xmlChar;
    if !safe_ctxt.extSubSystem.is_null()
        && (dict.is_null()
            || unsafe { xmlDictOwns_safe(dict, safe_ctxt.extSubSystem as *const xmlChar) } == 0)
    {
        unsafe {
            xmlFree_safe(safe_ctxt.extSubSystem as *mut i8 as *mut ());
        }
    }
    safe_ctxt.extSubSystem = 0 as *mut xmlChar;
    if !safe_ctxt.myDoc.is_null() {
        unsafe {
            xmlFreeDoc_safe(safe_ctxt.myDoc);
        }
    }
    safe_ctxt.myDoc = 0 as xmlDocPtr;
    safe_ctxt.standalone = -1;
    safe_ctxt.hasExternalSubset = 0;
    safe_ctxt.hasPErefs = 0;
    safe_ctxt.html = 0;
    safe_ctxt.external = 0;
    safe_ctxt.instate = XML_PARSER_START;
    safe_ctxt.token = 0;
    safe_ctxt.wellFormed = 1;
    safe_ctxt.nsWellFormed = 1;
    safe_ctxt.disableSAX = 0;
    safe_ctxt.valid = 1;
    safe_ctxt.record_info = 0;
    safe_ctxt.checkIndex = 0;
    safe_ctxt.inSubset = 0;
    safe_ctxt.errNo = XML_ERR_OK as i32;
    safe_ctxt.depth = 0;
    safe_ctxt.charset = XML_CHAR_ENCODING_UTF8 as i32;
    safe_ctxt.catalogs = 0 as *mut ();
    safe_ctxt.nbentities = 0;
    safe_ctxt.sizeentities = 0;
    safe_ctxt.sizeentcopy = 0;
    unsafe {
        xmlInitNodeInfoSeq_safe(&mut safe_ctxt.node_seq);
        if !safe_ctxt.attsDefault.is_null() {
            xmlHashFree_safe(
                safe_ctxt.attsDefault,
                Some(
                    xmlHashDefaultDeallocator
                        as unsafe extern "C" fn(_: *mut (), _: *const xmlChar) -> (),
                ),
            );
            safe_ctxt.attsDefault = 0 as xmlHashTablePtr
        }
    }
    if !safe_ctxt.attsSpecial.is_null() {
        unsafe {
            xmlHashFree_safe(safe_ctxt.attsSpecial, None);
        }
        safe_ctxt.attsSpecial = 0 as xmlHashTablePtr
    }

    match () {
        #[cfg(HAVE_parser_LIBXML_CATALOG_ENABLED)]
        _ => {
            if !safe_ctxt.catalogs.is_null() {
                unsafe {
                    xmlCatalogFreeLocal_safe(safe_ctxt.catalogs);
                }
            }
        }
        #[cfg(not(HAVE_parser_LIBXML_CATALOG_ENABLED))]
        _ => {}
    };
    if safe_ctxt.lastError.code != XML_ERR_OK as i32 {
        unsafe {
            xmlResetError_safe(&mut safe_ctxt.lastError);
        }
    };
}
/* *
* xmlCtxtResetPush:
* @ctxt: an XML parser context
* @chunk:  a pointer to an array of chars
* @size:  number of chars in the array
* @filename:  an optional file name or URI
* @encoding:  the document encoding, or NULL
*
* Reset a push parser context
*
* Returns 0 in case of success and 1 in case of error
*/

pub fn xmlCtxtResetPush(
    ctxt: xmlParserCtxtPtr,
    chunk: *const i8,
    size: i32,
    filename: *const i8,
    encoding: *const i8,
) -> i32 {
    let mut inputStream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    let mut buf: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    let mut enc: xmlCharEncoding = XML_CHAR_ENCODING_NONE;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if ctxt.is_null() {
        return 1;
    }
    if encoding.is_null() && !chunk.is_null() && size >= 4 {
        enc = unsafe { xmlDetectCharEncoding_safe(chunk as *const xmlChar, size) }
    }
    buf = unsafe { xmlAllocParserInputBuffer_safe(enc) };
    if buf.is_null() {
        return 1;
    }
    if ctxt.is_null() {
        unsafe {
            xmlFreeParserInputBuffer_safe(buf);
        }
        return 1;
    }
    unsafe {
        xmlCtxtReset_safe(ctxt);
    }
    if filename.is_null() {
        safe_ctxt.directory = 0 as *mut i8
    } else {
        safe_ctxt.directory = unsafe { xmlParserGetDirectory_safe(filename) }
    }
    inputStream = unsafe { xmlNewInputStream_safe(ctxt) };
    if inputStream.is_null() {
        unsafe {
            xmlFreeParserInputBuffer_safe(buf);
        }
        return 1;
    }
    let mut safe_inputStream = unsafe { &mut *inputStream };
    if filename.is_null() {
        safe_inputStream.filename = 0 as *const i8
    } else {
        safe_inputStream.filename =
            unsafe { xmlCanonicPath_safe(filename as *const xmlChar) as *mut i8 }
    }
    safe_inputStream.buf = buf;
    let mut safe_buf = unsafe { &mut *buf };
    unsafe {
        xmlBufResetInput_safe(safe_buf.buffer, inputStream);
        inputPush_safe(ctxt, inputStream);
    }
    if size > 0
        && !chunk.is_null()
        && !safe_ctxt.input.is_null()
        && unsafe { !(*safe_ctxt.input).buf.is_null() }
    {
        unsafe {
            let mut base: size_t =
                xmlBufGetInputBase_safe((*(*safe_ctxt.input).buf).buffer, safe_ctxt.input);
            let mut cur: size_t =
                (*safe_ctxt.input).cur.offset_from((*safe_ctxt.input).base) as i64 as size_t;
            xmlParserInputBufferPush_safe((*safe_ctxt.input).buf, size, chunk);
            xmlBufSetInputBaseCur_safe(
                (*(*safe_ctxt.input).buf).buffer,
                safe_ctxt.input,
                base,
                cur,
            );
        }

        match () {
            #[cfg(HAVE_parser_DEBUG_PUSH)]
            _ => {
                (*__xmlGenericError()).expect("non-null function pointer")(
                    *__xmlGenericErrorContext(),
                    b"PP: pushed %d\n\x00" as *const u8 as *const i8,
                    size,
                );
            }
            #[cfg(not(HAVE_parser_DEBUG_PUSH))]
            _ => {}
        };
    }
    if !encoding.is_null() {
        let mut hdlr: xmlCharEncodingHandlerPtr = 0 as *mut xmlCharEncodingHandler;
        if !safe_ctxt.encoding.is_null() {
            unsafe {
                xmlFree.expect("non-null function pointer")(
                    safe_ctxt.encoding as *mut xmlChar as *mut (),
                );
            }
        }
        safe_ctxt.encoding = unsafe { xmlStrdup_safe(encoding as *const xmlChar) };
        hdlr = unsafe { xmlFindCharEncodingHandler_safe(encoding) };
        if !hdlr.is_null() {
            unsafe { xmlSwitchToEncoding_safe(ctxt, hdlr) };
        } else {
            unsafe {
                xmlFatalErrMsgStr(
                    ctxt,
                    XML_ERR_UNSUPPORTED_ENCODING,
                    b"Unsupported encoding %s\n\x00" as *const u8 as *const i8,
                    encoding as *mut xmlChar,
                );
            }
        }
    } else if enc as i32 != XML_CHAR_ENCODING_NONE as i32 {
        unsafe { xmlSwitchEncoding_safe(ctxt, enc) };
    }
    return 0;
}
/* *
* xmlCtxtUseOptionsInternal:
* @ctxt: an XML parser context
* @options:  a combination of xmlParserOption
* @encoding:  the user provided encoding to use
*
* Applies the options to the parser context
*
* Returns 0 in case of success, the set of unknown or unimplemented options
*         in case of error.
*/
fn xmlCtxtUseOptionsInternal(ctxt: xmlParserCtxtPtr, mut options: i32, encoding: *const i8) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if ctxt.is_null() {
        return -1;
    }
    if !encoding.is_null() {
        if !safe_ctxt.encoding.is_null() {
            unsafe {
                xmlFree_safe(safe_ctxt.encoding as *mut xmlChar as *mut ());
            }
        }
        unsafe { safe_ctxt.encoding = xmlStrdup_safe(encoding as *const xmlChar) }
    }
    if options & XML_PARSE_RECOVER as i32 != 0 {
        safe_ctxt.recovery = 1;
        options -= XML_PARSE_RECOVER as i32;
        safe_ctxt.options |= XML_PARSE_RECOVER as i32
    } else {
        safe_ctxt.recovery = 0
    }
    if options & XML_PARSE_DTDLOAD as i32 != 0 {
        safe_ctxt.loadsubset = 2;
        options -= XML_PARSE_DTDLOAD as i32;
        safe_ctxt.options |= XML_PARSE_DTDLOAD as i32
    } else {
        safe_ctxt.loadsubset = 0
    }
    if options & XML_PARSE_DTDATTR as i32 != 0 {
        safe_ctxt.loadsubset |= 4;
        options -= XML_PARSE_DTDATTR as i32;
        safe_ctxt.options |= XML_PARSE_DTDATTR as i32
    }
    if options & XML_PARSE_NOENT as i32 != 0 {
        safe_ctxt.replaceEntities = 1;
        /* ctxt->loadsubset |= XML_DETECT_IDS; */
        options -= XML_PARSE_NOENT as i32;
        safe_ctxt.options |= XML_PARSE_NOENT as i32
    } else {
        safe_ctxt.replaceEntities = 0
    }
    if options & XML_PARSE_PEDANTIC as i32 != 0 {
        safe_ctxt.pedantic = 1;
        options -= XML_PARSE_PEDANTIC as i32;
        safe_ctxt.options |= XML_PARSE_PEDANTIC as i32
    } else {
        safe_ctxt.pedantic = 0
    }
    if options & XML_PARSE_NOBLANKS as i32 != 0 {
        safe_ctxt.keepBlanks = 0;
        unsafe {
            (*safe_ctxt.sax).ignorableWhitespace = Some(
                xmlSAX2IgnorableWhitespace
                    as unsafe extern "C" fn(_: *mut (), _: *const xmlChar, _: i32) -> (),
            )
        };
        options -= XML_PARSE_NOBLANKS as i32;
        safe_ctxt.options |= XML_PARSE_NOBLANKS as i32
    } else {
        safe_ctxt.keepBlanks = 1
    }
    if options & XML_PARSE_DTDVALID as i32 != 0 {
        safe_ctxt.validate = 1;
        if options & XML_PARSE_NOWARNING as i32 != 0 {
            safe_ctxt.vctxt.warning = None
        }
        if options & XML_PARSE_NOERROR as i32 != 0 {
            safe_ctxt.vctxt.error = None
        }
        options -= XML_PARSE_DTDVALID as i32;
        safe_ctxt.options |= XML_PARSE_DTDVALID as i32
    } else {
        safe_ctxt.validate = 0
    }
    unsafe {
        if options & XML_PARSE_NOWARNING as i32 != 0 {
            (*safe_ctxt.sax).warning = None;
            options -= XML_PARSE_NOWARNING as i32
        }
        if options & XML_PARSE_NOERROR as i32 != 0 {
            (*safe_ctxt.sax).error = None;
            (*safe_ctxt.sax).fatalError = None;
            options -= XML_PARSE_NOERROR as i32
        }
    }
    match () {
        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
        _ => unsafe {
            if options & XML_PARSE_SAX1 as i32 != 0 {
                (*safe_ctxt.sax).startElement = Some(
                    xmlSAX2StartElement
                        as unsafe extern "C" fn(
                            _: *mut (),
                            _: *const xmlChar,
                            _: *mut *const xmlChar,
                        ) -> (),
                );
                (*safe_ctxt.sax).endElement = Some(
                    xmlSAX2EndElement as unsafe extern "C" fn(_: *mut (), _: *const xmlChar) -> (),
                );
                (*safe_ctxt.sax).startElementNs = None;
                (*safe_ctxt.sax).endElementNs = None;
                (*safe_ctxt.sax).initialized = 1;
                options -= XML_PARSE_SAX1 as i32;
                safe_ctxt.options |= XML_PARSE_SAX1 as i32
            }
        },
        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
        _ => {}
    };
    /* LIBXML_SAX1_ENABLED */
    if options & XML_PARSE_NODICT as i32 != 0 {
        safe_ctxt.dictNames = 0;
        options -= XML_PARSE_NODICT as i32;
        safe_ctxt.options |= XML_PARSE_NODICT as i32
    } else {
        safe_ctxt.dictNames = 1
    }
    if options & XML_PARSE_NOCDATA as i32 != 0 {
        unsafe {
            (*safe_ctxt.sax).cdataBlock = None;
        }
        options -= XML_PARSE_NOCDATA as i32;
        safe_ctxt.options |= XML_PARSE_NOCDATA as i32
    }
    if options & XML_PARSE_NSCLEAN as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_NSCLEAN as i32;
        options -= XML_PARSE_NSCLEAN as i32
    }
    if options & XML_PARSE_NONET as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_NONET as i32;
        options -= XML_PARSE_NONET as i32
    }
    if options & XML_PARSE_COMPACT as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_COMPACT as i32;
        options -= XML_PARSE_COMPACT as i32
    }
    if options & XML_PARSE_OLD10 as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_OLD10 as i32;
        options -= XML_PARSE_OLD10 as i32
    }
    if options & XML_PARSE_NOBASEFIX as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_NOBASEFIX as i32;
        options -= XML_PARSE_NOBASEFIX as i32
    }
    if options & XML_PARSE_HUGE as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_HUGE as i32;
        options -= XML_PARSE_HUGE as i32;
        if !safe_ctxt.dict.is_null() {
            unsafe {
                xmlDictSetLimit_safe(safe_ctxt.dict, 0 as i32 as size_t);
            }
        }
    }
    if options & XML_PARSE_OLDSAX as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_OLDSAX as i32;
        options -= XML_PARSE_OLDSAX as i32
    }
    if options & XML_PARSE_IGNORE_ENC as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_IGNORE_ENC as i32;
        options -= XML_PARSE_IGNORE_ENC as i32
    }
    if options & XML_PARSE_BIG_LINES as i32 != 0 {
        safe_ctxt.options |= XML_PARSE_BIG_LINES as i32;
        options -= XML_PARSE_BIG_LINES as i32
    }
    safe_ctxt.linenumbers = 1;
    return options;
}
/* *
* xmlCtxtUseOptions:
* @ctxt: an XML parser context
* @options:  a combination of xmlParserOption
*
* Applies the options to the parser context
*
* Returns 0 in case of success, the set of unknown or unimplemented options
*         in case of error.
*/

pub fn xmlCtxtUseOptions(ctxt: xmlParserCtxtPtr, options: i32) -> i32 {
    return xmlCtxtUseOptionsInternal(ctxt, options, 0 as *const i8);
}
/* *
* xmlDoRead:
* @ctxt:  an XML parser context
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
* @reuse:  keep the context for reuse
*
* Common front-end for the xmlRead functions
*
* Returns the resulting document tree or NULL
*/

fn xmlDoRead(
    ctxt: xmlParserCtxtPtr,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
    reuse: i32,
) -> xmlDocPtr {
    let mut ret: xmlDocPtr = 0 as *mut xmlDoc;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    xmlCtxtUseOptionsInternal(ctxt, options, encoding);
    if !encoding.is_null() {
        let mut hdlr: xmlCharEncodingHandlerPtr = 0 as *mut xmlCharEncodingHandler;
        hdlr = unsafe { xmlFindCharEncodingHandler_safe(encoding) };
        if !hdlr.is_null() {
            unsafe {
                xmlSwitchToEncoding_safe(ctxt, hdlr);
            }
        }
    }
    unsafe {
        if !URL.is_null() && !safe_ctxt.input.is_null() && (*safe_ctxt.input).filename.is_null() {
            (*safe_ctxt.input).filename = xmlStrdup(URL as *const xmlChar) as *mut i8
        }
    };
    unsafe { xmlParseDocument(ctxt) };
    if safe_ctxt.wellFormed != 0 || safe_ctxt.recovery != 0 {
        ret = safe_ctxt.myDoc
    } else {
        ret = 0 as xmlDocPtr;
        if !safe_ctxt.myDoc.is_null() {
            unsafe {
                xmlFreeDoc_safe(safe_ctxt.myDoc);
            }
        }
    }
    safe_ctxt.myDoc = 0 as xmlDocPtr;
    if reuse == 0 {
        unsafe {
            xmlFreeParserCtxt_safe(ctxt);
        }
    }
    return ret;
}
/* *
* xmlReadDoc:
* @cur:  a pointer to a zero terminated string
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML in-memory document and build a tree.
*
* Returns the resulting document tree
*/

pub fn xmlReadDoc(
    cur: *const xmlChar,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    if cur.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
    }
    ctxt = xmlCreateDocParserCtxt(cur);
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    return xmlDoRead(ctxt, URL, encoding, options, 0 as i32);
}
/* *
* xmlReadFile:
* @filename:  a file or URL
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML file from the filesystem or the network.
*
* Returns the resulting document tree
*/

pub fn xmlReadFile(filename: *const i8, encoding: *const i8, options: i32) -> xmlDocPtr {
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    unsafe {
        xmlInitParser_safe();
    }
    ctxt = xmlCreateURLParserCtxt(filename, options);
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    return xmlDoRead(ctxt, 0 as *const i8, encoding, options, 0 as i32);
}
/* *
* xmlReadMemory:
* @buffer:  a pointer to a char array
* @size:  the size of the array
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML in-memory document and build a tree.
*
* Returns the resulting document tree
*/

pub fn xmlReadMemory(
    buffer: *const i8,
    size: i32,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut ctxt: xmlParserCtxtPtr = 0 as *mut xmlParserCtxt;
    unsafe {
        xmlInitParser_safe();
        ctxt = xmlCreateMemoryParserCtxt_safe(buffer, size);
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    return xmlDoRead(ctxt, URL, encoding, options, 0);
}
/* *
* xmlReadFd:
* @fd:  an open file descriptor
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML from a file descriptor and build a tree.
* NOTE that the file descriptor will not be closed when the
*      reader is closed or reset.
*
* Returns the resulting document tree
*/

pub fn xmlReadFd(fd: i32, URL: *const i8, encoding: *const i8, options: i32) -> xmlDocPtr {
    let ctxt: xmlParserCtxtPtr;
    let mut input: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    let stream: xmlParserInputPtr;
    let safe_input = unsafe { &mut *input };
    if fd < 0 {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        input = xmlParserInputBufferCreateFd_safe(fd, XML_CHAR_ENCODING_NONE);
    }
    if input.is_null() {
        return 0 as xmlDocPtr;
    }
    safe_input.closecallback = None;
    unsafe {
        ctxt = xmlNewParserCtxt_safe();
        if ctxt.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            return 0 as xmlDocPtr;
        }
        stream = xmlNewIOInputStream_safe(ctxt, input, XML_CHAR_ENCODING_NONE);
        if stream.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            xmlFreeParserCtxt_safe(ctxt);
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 0 as i32);
}
/* *
* xmlReadIO:
* @ioread:  an I/O read function
* @ioclose:  an I/O close function
* @ioctx:  an I/O handler
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML document from I/O functions and source and build a tree.
*
* Returns the resulting document tree
*/

pub fn xmlReadIO(
    ioread: xmlInputReadCallback,
    ioclose: xmlInputCloseCallback,
    ioctx: *mut (),
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut ctxt: xmlParserCtxtPtr;
    let mut input: xmlParserInputBufferPtr;
    let mut stream: xmlParserInputPtr;
    if ioread.is_none() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        input = xmlParserInputBufferCreateIO_safe(ioread, ioclose, ioctx, XML_CHAR_ENCODING_NONE);
    }
    if input.is_null() {
        unsafe {
            if ioclose.is_some() {
                ioclose.expect("non-null function pointer")(ioctx);
            }
        }
        return 0 as xmlDocPtr;
    }
    unsafe { ctxt = xmlNewParserCtxt_safe() };
    if ctxt.is_null() {
        unsafe { xmlFreeParserInputBuffer_safe(input) };
        return 0 as xmlDocPtr;
    }
    unsafe {
        stream = xmlNewIOInputStream_safe(ctxt, input, XML_CHAR_ENCODING_NONE);
        if stream.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            xmlFreeParserCtxt_safe(ctxt);
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 0);
}
/* *
* xmlCtxtReadDoc:
* @ctxt:  an XML parser context
* @cur:  a pointer to a zero terminated string
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML in-memory document and build a tree.
* This reuses the existing @ctxt parser context
*
* Returns the resulting document tree
*/

pub fn xmlCtxtReadDoc(
    ctxt: xmlParserCtxtPtr,
    cur: *const xmlChar,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut stream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    if cur.is_null() {
        return 0 as xmlDocPtr;
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        xmlCtxtReset_safe(ctxt);
        stream = xmlNewStringInputStream_safe(ctxt, cur);
        if stream.is_null() {
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 1);
}
/* *
* xmlCtxtReadFile:
* @ctxt:  an XML parser context
* @filename:  a file or URL
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML file from the filesystem or the network.
* This reuses the existing @ctxt parser context
*
* Returns the resulting document tree
*/

pub fn xmlCtxtReadFile(
    ctxt: xmlParserCtxtPtr,
    filename: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut stream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    if filename.is_null() {
        return 0 as xmlDocPtr;
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        xmlCtxtReset_safe(ctxt);
        stream = xmlLoadExternalEntity_safe(filename, 0 as *const i8, ctxt);
        if stream.is_null() {
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, 0 as *const i8, encoding, options, 1);
}
/* *
* xmlCtxtReadMemory:
* @ctxt:  an XML parser context
* @buffer:  a pointer to a char array
* @size:  the size of the array
* @URL:  the base URL to use for the document
* @encoding:  the document encoding, or NULL
* @options:  a combination of xmlParserOption
*
* parse an XML in-memory document and build a tree.
* This reuses the existing @ctxt parser context
*
* Returns the resulting document tree
*/

pub fn xmlCtxtReadMemory(
    ctxt: xmlParserCtxtPtr,
    buffer: *const i8,
    size: i32,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut input: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    let mut stream: xmlParserInputPtr = 0 as *mut xmlParserInput;
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    if buffer.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        xmlCtxtReset_safe(ctxt);
        input = xmlParserInputBufferCreateMem_safe(buffer, size, XML_CHAR_ENCODING_NONE);
    }
    if input.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        stream = xmlNewIOInputStream_safe(ctxt, input, XML_CHAR_ENCODING_NONE);
        if stream.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 1);
}

pub fn xmlCtxtReadFd(
    ctxt: xmlParserCtxtPtr,
    fd: i32,
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut input: xmlParserInputBufferPtr = 0 as *mut xmlParserInputBuffer;
    let mut stream: xmlParserInputPtr;
    let mut safe_input = unsafe { &mut *input };
    if fd < 0 {
        return 0 as xmlDocPtr;
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        xmlCtxtReset_safe(ctxt);
        input = xmlParserInputBufferCreateFd_safe(fd, XML_CHAR_ENCODING_NONE);
    }
    if input.is_null() {
        return 0 as xmlDocPtr;
    }
    safe_input.closecallback = None;
    unsafe {
        stream = xmlNewIOInputStream_safe(ctxt, input, XML_CHAR_ENCODING_NONE);
        if stream.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 1);
}

pub fn xmlCtxtReadIO(
    ctxt: xmlParserCtxtPtr,
    ioread: xmlInputReadCallback,
    ioclose: xmlInputCloseCallback,
    ioctx: *mut (),
    URL: *const i8,
    encoding: *const i8,
    options: i32,
) -> xmlDocPtr {
    let mut input: xmlParserInputBufferPtr;
    let mut stream: xmlParserInputPtr;
    if ioread.is_none() {
        return 0 as xmlDocPtr;
    }
    if ctxt.is_null() {
        return 0 as xmlDocPtr;
    }
    unsafe {
        xmlInitParser_safe();
        xmlCtxtReset_safe(ctxt);
        input = xmlParserInputBufferCreateIO_safe(ioread, ioclose, ioctx, XML_CHAR_ENCODING_NONE);
    }
    if input.is_null() {
        unsafe {
            if ioclose.is_some() {
                ioclose.expect("non-null function pointer")(ioctx);
            }
        }
        return 0 as xmlDocPtr;
    }
    unsafe {
        stream = xmlNewIOInputStream_safe(ctxt, input, XML_CHAR_ENCODING_NONE);
        if stream.is_null() {
            xmlFreeParserInputBuffer_safe(input);
            return 0 as xmlDocPtr;
        }
        inputPush_safe(ctxt, stream);
    }
    return xmlDoRead(ctxt, URL, encoding, options, 1);
}

fn xmlParserEntityCheck(
    ctxt: xmlParserCtxtPtr,
    mut size: size_t,
    ent: xmlEntityPtr,
    replacement: size_t,
) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut safe_ent = unsafe { &mut *ent };

    let mut consumed: size_t = 0;
    let mut i: i32 = 0;
    if ctxt.is_null() || (safe_ctxt).options & XML_PARSE_HUGE as i32 != 0 {
        return 0;
    }
    if (safe_ctxt).lastError.code == XML_ERR_ENTITY_LOOP as i32 {
        return 1;
    }
    /*
     * This may look absurd but is needed to detect
     * entities problems
     */
    if !ent.is_null()
        && (safe_ent).etype as u32 != XML_INTERNAL_PREDEFINED_ENTITY as i32 as u32
        && !(safe_ent).content.is_null()
        && (safe_ent).checked == 0
        && (safe_ctxt).errNo != XML_ERR_ENTITY_LOOP as i32
    {
        let mut oldnbent: u64 = (safe_ctxt).nbentities;
        let mut diff: u64 = 0;
        let mut rep: *mut xmlChar = 0 as *mut xmlChar;
        (safe_ent).checked = 1;
        (safe_ctxt).depth += 1;
        unsafe {
            rep = xmlStringDecodeEntities(ctxt, (safe_ent).content, 1, 0, 0, 0);
        }
        (safe_ctxt).depth -= 1;
        if rep.is_null() || (safe_ctxt).errNo == XML_ERR_ENTITY_LOOP as i32 {
            unsafe {
                *(safe_ent).content.offset(0) = 0 as i32 as xmlChar;
            }
        }
        diff = (safe_ctxt)
            .nbentities
            .wrapping_sub(oldnbent)
            .wrapping_add(1);
        if diff > (i32::MAX / 2) as u64 {
            diff = (i32::MAX / 2) as u64
        }
        (safe_ent).checked = diff.wrapping_mul(2) as i32;
        unsafe {
            if !rep.is_null() {
                if !xmlStrchr_safe(rep, '<' as i32 as xmlChar).is_null() {
                    (safe_ent).checked |= 1 as i32
                }
                xmlFree_safe(rep as *mut ());
                rep = 0 as *mut xmlChar
            }
        }
    }
    /*
     * Prevent entity exponential check, not just replacement while
     * parsing the DTD
     * The check is potentially costly so do that only once in a thousand
     */
    if (safe_ctxt).instate as i32 == XML_PARSER_DTD as i32
        && (safe_ctxt).nbentities > 10000
        && (safe_ctxt).nbentities.wrapping_rem(1024) == 0 as i32 as u64
    {
        i = 0;
        while i < (safe_ctxt).inputNr {
            consumed = unsafe {
                (consumed as u64).wrapping_add(
                    (**(safe_ctxt).inputTab.offset(i as isize))
                        .consumed
                        .wrapping_add(
                            (**(safe_ctxt).inputTab.offset(i as isize))
                                .cur
                                .offset_from((**(safe_ctxt).inputTab.offset(i as isize)).base)
                                as i64 as u64,
                        ),
                ) as size_t as size_t
            };
            i += 1
        }
        if (safe_ctxt).nbentities > consumed.wrapping_mul(10) {
            unsafe {
                xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
            }
            (safe_ctxt).instate = XML_PARSER_EOF;
            return 1;
        }
        consumed = 0
    }
    if replacement != 0 {
        if replacement < 10000000 {
            return 0;
        }
        /*
         * If the volume of entity copy reaches 10 times the
         * amount of parsed data and over the large text threshold
         * then that's very likely to be an abuse.
         */
        if !(safe_ctxt).input.is_null() {
            consumed = unsafe {
                (*(safe_ctxt).input).consumed.wrapping_add(
                    (*(safe_ctxt).input)
                        .cur
                        .offset_from((*(safe_ctxt).input).base) as i64 as u64,
                )
            }
        }
        consumed = (consumed as u64).wrapping_add((safe_ctxt).sizeentities) as size_t as size_t;
        if replacement < (10 as i32 as u64).wrapping_mul(consumed) {
            return 0;
        }
    } else if size != 0 {
        /*
         * Do the check based on the replacement size of the entity
         */
        if size < 1000 {
            return 0;
        }
        /*
         * A limit on the amount of text data reasonably used
         */
        if !(safe_ctxt).input.is_null() {
            consumed = unsafe {
                (*(safe_ctxt).input).consumed.wrapping_add(
                    (*(safe_ctxt).input)
                        .cur
                        .offset_from((*(safe_ctxt).input).base) as i64 as u64,
                )
            }
        }
        consumed = (consumed as u64).wrapping_add((safe_ctxt).sizeentities) as size_t as size_t;
        if size < (10 as i32 as u64).wrapping_mul(consumed)
            && (safe_ctxt).nbentities.wrapping_mul(3) < (10 as i32 as u64).wrapping_mul(consumed)
        {
            return 0;
        }
    } else if !ent.is_null() {
        /*
         * use the number of parsed entities in the replacement
         */
        size = ((safe_ent).checked / 2) as size_t;
        /*
         * The amount of data parsed counting entities size only once
         */
        if !(safe_ctxt).input.is_null() {
            consumed = unsafe {
                (*(safe_ctxt).input).consumed.wrapping_add(
                    (*(safe_ctxt).input)
                        .cur
                        .offset_from((*(safe_ctxt).input).base) as i64 as u64,
                )
            }
        }
        consumed = (consumed as u64).wrapping_add((safe_ctxt).sizeentities) as size_t as size_t;
        /*
         * Check the density of entities for the amount of data
         * knowing an entity reference will take at least 3 bytes
         */
        if size.wrapping_mul(3) < consumed.wrapping_mul(10) {
            return 0;
        }
    } else if (safe_ctxt).lastError.code != XML_ERR_UNDECLARED_ENTITY as i32
        && (safe_ctxt).lastError.code != XML_WAR_UNDECLARED_ENTITY as i32
        || (safe_ctxt).nbentities <= 10000
    {
        return 0;
    }
    unsafe {
        xmlFatalErr(ctxt, XML_ERR_ENTITY_LOOP, 0 as *const i8);
    }
    return 1;
}

pub static mut xmlParserMaxDepth: u32 = 256 as i32 as u32;
/*
* List of XML prefixed PI allowed by W3C specs
*/
static mut xmlW3CPIs: [*const i8; 3] = [
    b"xml-stylesheet\x00" as *const u8 as *const i8,
    b"xml-model\x00" as *const u8 as *const i8,
    0 as *const i8,
];
/* ***********************************************************************
*									*
*		Some factorized error routines				*
*									*
************************************************************************/
/* *
* xmlErrAttributeDup:
* @ctxt:  an XML parser context
* @prefix:  the attribute prefix
* @localname:  the attribute localname
*
* Handle a redefinition of attribute error
*/
fn xmlErrAttributeDup(ctxt: xmlParserCtxtPtr, prefix: *const xmlChar, localname: *const xmlChar) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = XML_ERR_ATTRIBUTE_REDEFINED as i32
    }
    if prefix.is_null() {
        unsafe {
            __xmlRaiseError(
                None,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                XML_ERR_ATTRIBUTE_REDEFINED as i32,
                XML_ERR_FATAL,
                0 as *const i8,
                0 as i32,
                localname as *const i8,
                0 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                b"Attribute %s redefined\n\x00" as *const u8 as *const i8,
                localname,
            );
        }
    } else {
        unsafe {
            __xmlRaiseError(
                None,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                XML_ERR_ATTRIBUTE_REDEFINED as i32,
                XML_ERR_FATAL,
                0 as *const i8,
                0 as i32,
                prefix as *const i8,
                localname as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                b"Attribute %s:%s redefined\n\x00" as *const u8 as *const i8,
                prefix,
                localname,
            );
        }
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlFatalErr:
* @ctxt:  an XML parser context
* @error:  the error number
* @extra:  extra information string
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
unsafe fn xmlFatalErr(mut ctxt: xmlParserCtxtPtr, mut error: xmlParserErrors, mut info: *const i8) {
    let mut safe_ctxt = unsafe { &mut *ctxt };

    let mut errmsg: *const i8 = 0 as *const i8;
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0 as i32
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    match error as u32 {
        XML_ERR_INVALID_HEX_CHARREF => {
            errmsg = b"CharRef: invalid hexadecimal value\x00" as *const u8 as *const i8
        }
        XML_ERR_INVALID_DEC_CHARREF => {
            errmsg = b"CharRef: invalid decimal value\x00" as *const u8 as *const i8
        }
        XML_ERR_INVALID_CHARREF => errmsg = b"CharRef: invalid value\x00" as *const u8 as *const i8,
        XML_ERR_INTERNAL_ERROR => errmsg = b"internal error\x00" as *const u8 as *const i8,
        XML_ERR_PEREF_AT_EOF => {
            errmsg = b"PEReference at end of document\x00" as *const u8 as *const i8
        }
        XML_ERR_PEREF_IN_PROLOG => errmsg = b"PEReference in prolog\x00" as *const u8 as *const i8,
        XML_ERR_PEREF_IN_EPILOG => errmsg = b"PEReference in epilog\x00" as *const u8 as *const i8,
        XML_ERR_PEREF_NO_NAME => errmsg = b"PEReference: no name\x00" as *const u8 as *const i8,
        XML_ERR_PEREF_SEMICOL_MISSING => {
            errmsg = b"PEReference: expecting \';\'\x00" as *const u8 as *const i8
        }
        XML_ERR_ENTITY_LOOP => {
            errmsg = b"Detected an entity reference loop\x00" as *const u8 as *const i8
        }
        XML_ERR_ENTITY_NOT_STARTED => {
            errmsg = b"EntityValue: \" or \' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_ENTITY_PE_INTERNAL => {
            errmsg = b"PEReferences forbidden in internal subset\x00" as *const u8 as *const i8
        }
        XML_ERR_ENTITY_NOT_FINISHED => {
            errmsg = b"EntityValue: \" or \' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_ATTRIBUTE_NOT_STARTED => {
            errmsg = b"AttValue: \" or \' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_LT_IN_ATTRIBUTE => {
            errmsg =
                b"Unescaped \'<\' not allowed in attributes values\x00" as *const u8 as *const i8
        }
        XML_ERR_LITERAL_NOT_STARTED => {
            errmsg = b"SystemLiteral \" or \' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_LITERAL_NOT_FINISHED => {
            errmsg =
                b"Unfinished System or Public ID \" or \' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_MISPLACED_CDATA_END => {
            errmsg = b"Sequence \']]>\' not allowed in content\x00" as *const u8 as *const i8
        }
        XML_ERR_URI_REQUIRED => {
            errmsg = b"SYSTEM or PUBLIC, the URI is missing\x00" as *const u8 as *const i8
        }
        XML_ERR_PUBID_REQUIRED => {
            errmsg = b"PUBLIC, the Public Identifier is missing\x00" as *const u8 as *const i8
        }
        XML_ERR_HYPHEN_IN_COMMENT => {
            errmsg =
                b"Comment must not contain \'--\' (double-hyphen)\x00" as *const u8 as *const i8
        }
        XML_ERR_PI_NOT_STARTED => {
            errmsg = b"xmlParsePI : no target name\x00" as *const u8 as *const i8
        }
        XML_ERR_RESERVED_XML_NAME => errmsg = b"Invalid PI name\x00" as *const u8 as *const i8,
        XML_ERR_NOTATION_NOT_STARTED => {
            errmsg = b"NOTATION: Name expected here\x00" as *const u8 as *const i8
        }
        XML_ERR_NOTATION_NOT_FINISHED => {
            errmsg = b"\'>\' required to close NOTATION declaration\x00" as *const u8 as *const i8
        }
        XML_ERR_VALUE_REQUIRED => errmsg = b"Entity value required\x00" as *const u8 as *const i8,
        XML_ERR_URI_FRAGMENT => errmsg = b"Fragment not allowed\x00" as *const u8 as *const i8,
        XML_ERR_ATTLIST_NOT_STARTED => {
            errmsg = b"\'(\' required to start ATTLIST enumeration\x00" as *const u8 as *const i8
        }
        XML_ERR_NMTOKEN_REQUIRED => {
            errmsg = b"NmToken expected in ATTLIST enumeration\x00" as *const u8 as *const i8
        }
        XML_ERR_ATTLIST_NOT_FINISHED => {
            errmsg = b"\')\' required to finish ATTLIST enumeration\x00" as *const u8 as *const i8
        }
        XML_ERR_MIXED_NOT_STARTED => {
            errmsg = b"MixedContentDecl : \'|\' or \')*\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_PCDATA_REQUIRED => {
            errmsg = b"MixedContentDecl : \'#PCDATA\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_ELEMCONTENT_NOT_STARTED => {
            errmsg = b"ContentDecl : Name or \'(\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_ELEMCONTENT_NOT_FINISHED => {
            errmsg = b"ContentDecl : \',\' \'|\' or \')\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_PEREF_IN_INT_SUBSET => {
            errmsg = b"PEReference: forbidden within markup decl in internal subset\x00"
                as *const u8 as *const i8
        }
        XML_ERR_GT_REQUIRED => errmsg = b"expected \'>\'\x00" as *const u8 as *const i8,
        XML_ERR_CONDSEC_INVALID => {
            errmsg = b"XML conditional section \'[\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_EXT_SUBSET_NOT_FINISHED => {
            errmsg = b"Content error in the external subset\x00" as *const u8 as *const i8
        }
        XML_ERR_CONDSEC_INVALID_KEYWORD => {
            errmsg = b"conditional section INCLUDE or IGNORE keyword expected\x00" as *const u8
                as *const i8
        }
        XML_ERR_CONDSEC_NOT_FINISHED => {
            errmsg = b"XML conditional section not closed\x00" as *const u8 as *const i8
        }
        XML_ERR_XMLDECL_NOT_STARTED => {
            errmsg = b"Text declaration \'<?xml\' required\x00" as *const u8 as *const i8
        }
        XML_ERR_XMLDECL_NOT_FINISHED => {
            errmsg = b"parsing XML declaration: \'?>\' expected\x00" as *const u8 as *const i8
        }
        XML_ERR_EXT_ENTITY_STANDALONE => {
            errmsg = b"external parsed entities cannot be standalone\x00" as *const u8 as *const i8
        }
        XML_ERR_ENTITYREF_SEMICOL_MISSING => {
            errmsg = b"EntityRef: expecting \';\'\x00" as *const u8 as *const i8
        }
        XML_ERR_DOCTYPE_NOT_FINISHED => {
            errmsg = b"DOCTYPE improperly terminated\x00" as *const u8 as *const i8
        }
        XML_ERR_LTSLASH_REQUIRED => {
            errmsg = b"EndTag: \'</\' not found\x00" as *const u8 as *const i8
        }
        XML_ERR_EQUAL_REQUIRED => errmsg = b"expected \'=\'\x00" as *const u8 as *const i8,
        XML_ERR_STRING_NOT_CLOSED => {
            errmsg = b"String not closed expecting \" or \'\x00" as *const u8 as *const i8
        }
        XML_ERR_STRING_NOT_STARTED => {
            errmsg = b"String not started expecting \' or \"\x00" as *const u8 as *const i8
        }
        XML_ERR_ENCODING_NAME => {
            errmsg = b"Invalid XML encoding name\x00" as *const u8 as *const i8
        }
        XML_ERR_STANDALONE_VALUE => {
            errmsg = b"standalone accepts only \'yes\' or \'no\'\x00" as *const u8 as *const i8
        }
        XML_ERR_DOCUMENT_EMPTY => errmsg = b"Document is empty\x00" as *const u8 as *const i8,
        XML_ERR_DOCUMENT_END => {
            errmsg = b"Extra content at the end of the document\x00" as *const u8 as *const i8
        }
        XML_ERR_NOT_WELL_BALANCED => {
            errmsg = b"chunk is not well balanced\x00" as *const u8 as *const i8
        }
        XML_ERR_EXTRA_CONTENT => {
            errmsg =
                b"extra content at the end of well balanced chunk\x00" as *const u8 as *const i8
        }
        XML_ERR_VERSION_MISSING => {
            errmsg = b"Malformed declaration expecting version\x00" as *const u8 as *const i8
        }
        XML_ERR_NAME_TOO_LONG => {
            errmsg = b"Name too long use XML_PARSE_HUGE option\x00" as *const u8 as *const i8
        }
        _ => errmsg = b"Unregistered error message\x00" as *const u8 as *const i8,
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    if info.is_null() {
        unsafe {
            __xmlRaiseError(
                None,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                error as i32,
                XML_ERR_FATAL,
                0 as *const i8,
                0 as i32,
                info,
                0 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                b"%s\n\x00" as *const u8 as *const i8,
                errmsg,
            );
        }
    } else {
        unsafe {
            __xmlRaiseError(
                None,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                error as i32,
                XML_ERR_FATAL,
                0 as *const i8,
                0 as i32,
                info,
                0 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                b"%s: %s\n\x00" as *const u8 as *const i8,
                errmsg,
                info,
            );
        }
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlFatalErrMsg:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
fn xmlFatalErrMsg(ctxt: xmlParserCtxtPtr, error: xmlParserErrors, msg: *const i8) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0 as i32
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_PARSER as i32,
            error as i32,
            XML_ERR_FATAL,
            0 as *const i8,
            0 as i32,
            0 as *const i8,
            0 as *const i8,
            0 as *const i8,
            0 as i32,
            0 as i32,
            b"%s\x00" as *const u8 as *const i8,
            msg,
        );
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlWarningMsg:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @str1:  extra data
* @str2:  extra data
*
* Handle a warning.
*/
fn xmlWarningMsg(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    str1: *const xmlChar,
    str2: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut schannel: xmlStructuredErrorFunc = None;
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null()
        && !(safe_ctxt).sax.is_null()
        && unsafe { (*(safe_ctxt).sax).initialized == 0xdeedbeaf as u32 }
    {
        schannel = unsafe { (*(safe_ctxt).sax).serror };
    }
    if !ctxt.is_null() {
        unsafe {
            __xmlRaiseError(
                schannel,
                if !(safe_ctxt).sax.is_null() {
                    (*(safe_ctxt).sax).warning
                } else {
                    None
                },
                (safe_ctxt).userData,
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                error as i32,
                XML_ERR_WARNING,
                0 as *const i8,
                0 as i32,
                str1 as *const i8,
                str2 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                msg,
                str1 as *const i8,
                str2 as *const i8,
            );
        }
    } else {
        unsafe {
            __xmlRaiseError(
                schannel,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_PARSER as i32,
                error as i32,
                XML_ERR_WARNING,
                0 as *const i8,
                0 as i32,
                str1 as *const i8,
                str2 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                msg,
                str1 as *const i8,
                str2 as *const i8,
            );
        }
    };
}
/* *
* xmlValidityError:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @str1:  extra data
*
* Handle a validity error.
*/
fn xmlValidityError(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    str1: *const xmlChar,
    str2: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut schannel: xmlStructuredErrorFunc = None;
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32;
        if !(safe_ctxt).sax.is_null()
            && unsafe { (*(safe_ctxt).sax).initialized == 0xdeedbeaf as u32 }
        {
            schannel = unsafe { (*(safe_ctxt).sax).serror };
        }
    }
    if !ctxt.is_null() {
        unsafe {
            __xmlRaiseError(
                schannel,
                (safe_ctxt).vctxt.error,
                (safe_ctxt).vctxt.userData,
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_DTD as i32,
                error as i32,
                XML_ERR_ERROR,
                0 as *const i8,
                0 as i32,
                str1 as *const i8,
                str2 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                msg,
                str1 as *const i8,
                str2 as *const i8,
            );
        }
        (safe_ctxt).valid = 0 as i32
    } else {
        unsafe {
            __xmlRaiseError(
                schannel,
                None,
                0 as *mut (),
                ctxt as *mut (),
                0 as *mut (),
                XML_FROM_DTD as i32,
                error as i32,
                XML_ERR_ERROR,
                0 as *const i8,
                0 as i32,
                str1 as *const i8,
                str2 as *const i8,
                0 as *const i8,
                0 as i32,
                0 as i32,
                msg,
                str1 as *const i8,
                str2 as *const i8,
            );
        }
    };
}
/* *
* xmlFatalErrMsgInt:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @val:  an integer value
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
fn xmlFatalErrMsgInt(ctxt: xmlParserCtxtPtr, error: xmlParserErrors, msg: *const i8, val: i32) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_PARSER as i32,
            error as i32,
            XML_ERR_FATAL,
            0 as *const i8,
            0 as i32,
            0 as *const i8,
            0 as *const i8,
            0 as *const i8,
            val,
            0 as i32,
            msg,
            val,
        );
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlFatalErrMsgStrIntStr:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @str1:  an string info
* @val:  an integer value
* @str2:  an string info
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
fn xmlFatalErrMsgStrIntStr(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    str1: *const xmlChar,
    val: i32,
    str2: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_PARSER as i32,
            error as i32,
            XML_ERR_FATAL,
            0 as *const i8,
            0 as i32,
            str1 as *const i8,
            str2 as *const i8,
            0 as *const i8,
            val,
            0 as i32,
            msg,
            str1,
            val,
            str2,
        );
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlFatalErrMsgStr:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @val:  a string value
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
fn xmlFatalErrMsgStr(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    val: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_PARSER as i32,
            error as i32,
            XML_ERR_FATAL,
            0 as *const i8,
            0 as i32,
            val as *const i8,
            0 as *const i8,
            0 as *const i8,
            0 as i32,
            0 as i32,
            msg,
            val,
        );
    }
    if !ctxt.is_null() {
        (safe_ctxt).wellFormed = 0;
        if (safe_ctxt).recovery == 0 {
            (safe_ctxt).disableSAX = 1
        }
    };
}
/* *
* xmlErrMsgStr:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the error message
* @val:  a string value
*
* Handle a non fatal parser error
*/
fn xmlErrMsgStr(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    val: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_PARSER as i32,
            error as i32,
            XML_ERR_ERROR,
            0 as *const i8,
            0 as i32,
            val as *const i8,
            0 as *const i8,
            0 as *const i8,
            0 as i32,
            0 as i32,
            msg,
            val,
        );
    }
}
/* *
* xmlNsErr:
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the message
* @info1:  extra information string
* @info2:  extra information string
*
* Handle a fatal parser error, i.e. violating Well-Formedness constraints
*/
fn xmlNsErr(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    info1: *const xmlChar,
    info2: *const xmlChar,
    info3: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    if !ctxt.is_null() {
        (safe_ctxt).errNo = error as i32
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_NAMESPACE as i32,
            error as i32,
            XML_ERR_ERROR,
            0 as *const i8,
            0 as i32,
            info1 as *const i8,
            info2 as *const i8,
            info3 as *const i8,
            0 as i32,
            0 as i32,
            msg,
            info1,
            info2,
            info3,
        );
    }
    if !ctxt.is_null() {
        (safe_ctxt).nsWellFormed = 0
    };
}
/* *
* xmlNsWarn
* @ctxt:  an XML parser context
* @error:  the error number
* @msg:  the message
* @info1:  extra information string
* @info2:  extra information string
*
* Handle a namespace warning error
*/
fn xmlNsWarn(
    ctxt: xmlParserCtxtPtr,
    error: xmlParserErrors,
    msg: *const i8,
    info1: *const xmlChar,
    info2: *const xmlChar,
    info3: *const xmlChar,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if !ctxt.is_null()
        && (safe_ctxt).disableSAX != 0
        && (safe_ctxt).instate as i32 == XML_PARSER_EOF as i32
    {
        return;
    }
    unsafe {
        __xmlRaiseError(
            None,
            None,
            0 as *mut (),
            ctxt as *mut (),
            0 as *mut (),
            XML_FROM_NAMESPACE as i32,
            error as i32,
            XML_ERR_WARNING,
            0 as *const i8,
            0 as i32,
            info1 as *const i8,
            info2 as *const i8,
            info3 as *const i8,
            0 as i32,
            0 as i32,
            msg,
            info1,
            info2,
            info3,
        );
    }
}
/* ***********************************************************************
*									*
*		Library wide options					*
*									*
************************************************************************/
/* *
* xmlHasFeature:
* @feature: the feature to be examined
*
* Examines if the library has been compiled with a given feature.
*
* Returns a non-zero value if the feature exist, otherwise zero.
* Returns zero (0) if the feature does not exist or an unknown
* unknown feature is requested, non-zero otherwise.
*/
//todo:
pub unsafe fn xmlHasFeature(mut feature: xmlFeature) -> i32 {
    match feature as u32 {
        1 => return 1 as i32,
        2 => return 1 as i32,
        3 => return 1 as i32,
        4 => return 1 as i32,
        5 => return 1 as i32,
        6 => return 1 as i32,
        7 => return 1 as i32,
        8 => return 1 as i32,
        9 => return 1 as i32,
        10 => return 1 as i32,
        11 => return 1 as i32,
        12 => return 1 as i32,
        13 => return 1 as i32,
        14 => return 1 as i32,
        15 => return 1 as i32,
        16 => return 1 as i32,
        17 => return 1 as i32,
        18 => return 1 as i32,
        19 => return 1 as i32,
        20 => return 1 as i32,
        21 => return 1 as i32,
        22 => return 1 as i32,
        23 => return 1 as i32,
        24 => return 0 as i32,
        25 => return 1 as i32,
        26 => return 1 as i32,
        27 => return 1 as i32,
        28 => return 1 as i32,
        29 => return 0 as i32,
        30 => return 0 as i32,
        31 => return 1 as i32,
        33 => return 1 as i32,
        32 => return 0 as i32,
        _ => {}
    }
    return 0 as i32;
}
/* ***********************************************************************
*									*
*		SAX2 defaulted attributes handling			*
*									*
************************************************************************/
/* *
* xmlDetectSAX2:
* @ctxt:  an XML parser context
*
* Do the SAX2 detection and specific initialization
*/
fn xmlDetectSAX2(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut sax: xmlSAXHandlerPtr = 0 as *mut xmlSAXHandler;
    if ctxt.is_null() {
        return;
    }
    sax = (safe_ctxt).sax;
    let mut safe_sax = unsafe { &mut *sax };
    match () {
        #[cfg(HAVE_parser_LIBXML_SAX1_ENABLED)]
        _ => {
            if !sax.is_null()
                && (safe_sax).initialized == 0xdeedbeaf as u32
                && ((safe_sax).startElementNs.is_some()
                    || (safe_sax).endElementNs.is_some()
                    || (safe_sax).startElement.is_none() && (safe_sax).endElement.is_none())
            {
                (safe_ctxt).sax2 = 1;
            }
        }
        #[cfg(not(HAVE_parser_LIBXML_SAX1_ENABLED))]
        _ => {
            (safe_ctxt).sax2 = 1 as i32;
        }
    };

    /* LIBXML_SAX1_ENABLED */
    unsafe {
        (safe_ctxt).str_xml = xmlDictLookup_safe(
            (safe_ctxt).dict,
            b"xml\x00" as *const u8 as *const i8 as *mut xmlChar,
            3,
        );
        (safe_ctxt).str_xmlns = xmlDictLookup_safe(
            (safe_ctxt).dict,
            b"xmlns\x00" as *const u8 as *const i8 as *mut xmlChar,
            5,
        );
        (safe_ctxt).str_xml_ns = xmlDictLookup_safe(
            (safe_ctxt).dict,
            b"http://www.w3.org/XML/1998/namespace\x00" as *const u8 as *const i8 as *const xmlChar,
            36,
        );
    }
    if (safe_ctxt).str_xml.is_null()
        || (safe_ctxt).str_xmlns.is_null()
        || (safe_ctxt).str_xml_ns.is_null()
    {
        unsafe { xmlErrMemory(ctxt, 0 as *const i8) };
    };
}
/* array of localname/prefix/values/external */
/* *
* xmlAttrNormalizeSpace:
* @src: the source string
* @dst: the target string
*
* Normalize the space in non CDATA attribute values:
* If the attribute type is not CDATA, then the XML processor MUST further
* process the normalized attribute value by discarding any leading and
* trailing space (#x20) characters, and by replacing sequences of space
* (#x20) characters by a single space (#x20) character.
* Note that the size of dst need to be at least src, and if one doesn't need
* to preserve dst (and it doesn't come from a dictionary or read-only) then
* passing src as dst is just fine.
*
* Returns a pointer to the normalized value (dst) or NULL if no conversion
*         is needed.
*/
fn xmlAttrNormalizeSpace(mut src: *const xmlChar, mut dst: *mut xmlChar) -> *mut xmlChar {
    if src.is_null() || dst.is_null() {
        return 0 as *mut xmlChar;
    }
    while unsafe { *src } as i32 == 0x20 as i32 {
        src = unsafe { src.offset(1) }
    }
    while unsafe { *src } as i32 != 0 as i32 {
        if unsafe { *src } as i32 == 0x20 as i32 {
            while unsafe { *src } as i32 == 0x20 as i32 {
                src = unsafe { src.offset(1) }
            }
            if unsafe { *src } as i32 != 0 as i32 {
                let fresh0 = dst;
                dst = unsafe { dst.offset(1) };
                unsafe { *fresh0 = 0x20 as i32 as xmlChar };
            }
        } else {
            let fresh1 = src;
            src = unsafe { src.offset(1) };
            let fresh2 = dst;
            dst = unsafe { dst.offset(1) };
            unsafe { *fresh2 = *fresh1 };
        }
    }
    unsafe { *dst = 0 as i32 as xmlChar };
    if dst == src as *mut xmlChar {
        return 0 as *mut xmlChar;
    }
    return dst;
}
/* *
* xmlAttrNormalizeSpace2:
* @src: the source string
*
* Normalize the space in non CDATA attribute values, a slightly more complex
* front end to avoid allocation problems when running on attribute values
* coming from the input.
*
* Returns a pointer to the normalized value (dst) or NULL if no conversion
*         is needed.
*/
fn xmlAttrNormalizeSpace2(
    ctxt: xmlParserCtxtPtr,
    src: *mut xmlChar,
    len: *mut i32,
) -> *const xmlChar {
    let mut i: i32 = 0;
    let mut remove_head: i32 = 0 as i32;
    let mut need_realloc: i32 = 0 as i32;
    let mut cur: *const xmlChar = 0 as *const xmlChar;
    if ctxt.is_null() || src.is_null() || len.is_null() {
        return 0 as *const xmlChar;
    }
    i = unsafe { *len };
    if i <= 0 {
        return 0 as *const xmlChar;
    }
    cur = src;
    while unsafe { *cur } as i32 == 0x20 {
        cur = unsafe { cur.offset(1) };
        remove_head += 1
    }
    while unsafe { *cur } as i32 != 0 {
        if unsafe { *cur } as i32 == 0x20 {
            cur = unsafe { cur.offset(1) };
            if !(unsafe { *cur } as i32 == 0x20 || unsafe { *cur } as i32 == 0) {
                continue;
            }
            need_realloc = 1;
            break;
        } else {
            cur = unsafe { cur.offset(1) }
        }
    }
    if need_realloc != 0 {
        let mut ret: *mut xmlChar = 0 as *mut xmlChar;
        ret = unsafe {
            xmlStrndup_safe(src.offset(remove_head as isize), i - remove_head + 1 as i32)
        };
        if ret.is_null() {
            unsafe { xmlErrMemory(ctxt, 0 as *const i8) };
            return 0 as *const xmlChar;
        }
        xmlAttrNormalizeSpace(ret, ret);
        unsafe { *len = strlen_safe(ret as *const i8) as i32 };
        return ret;
    } else {
        if remove_head != 0 {
            unsafe { *len -= remove_head };
            unsafe {
                memmove_safe(
                    src as *mut (),
                    src.offset(remove_head as isize) as *const (),
                    (1 + *len) as u64,
                );
            }
            return src;
        }
    }
    return 0 as *const xmlChar;
}
/* *
* xmlAddDefAttrs:
* @ctxt:  an XML parser context
* @fullname:  the element fullname
* @fullattr:  the attribute fullname
* @value:  the attribute value
*
* Add a defaulted attribute for an element
*/
fn xmlAddDefAttrs(
    ctxt: xmlParserCtxtPtr,
    fullname: *const xmlChar,
    fullattr: *const xmlChar,
    mut value: *const xmlChar,
) {
    let mut current_block: u64;
    let mut defaults: xmlDefAttrsPtr = 0 as *mut xmlDefAttrs;
    let mut len: i32 = 0;
    let mut name: *const xmlChar = 0 as *const xmlChar;
    let mut prefix: *const xmlChar = 0 as *const xmlChar;
    /*
     * Allows to detect attribute redefinitions
     */
    if unsafe { !(*ctxt).attsSpecial.is_null() } {
        unsafe {
            if !xmlHashLookup2_safe(unsafe { (*ctxt).attsSpecial }, fullname, fullattr).is_null() {
                return;
            }
        }
    }
    if unsafe { (*ctxt).attsDefault.is_null() } {
        unsafe { (*ctxt).attsDefault = xmlHashCreateDict_safe(10 as i32, (*ctxt).dict) };
        if unsafe { (*ctxt).attsDefault.is_null() } {
            current_block = 2968889880470072775;
        } else {
            current_block = 13183875560443969876;
        }
    } else {
        current_block = 13183875560443969876;
    }
    //@todo 削减unsafe范围
    unsafe {
        match current_block {
            13183875560443969876 => {
                /*
                 * split the element name into prefix:localname , the string found
                 * are within the DTD and then not associated to namespace names.
                 */
                name = xmlSplitQName3(fullname, &mut len);
                if name.is_null() {
                    name = xmlDictLookup_safe((*ctxt).dict, fullname, -(1 as i32));
                    prefix = 0 as *const xmlChar
                } else {
                    name = xmlDictLookup_safe((*ctxt).dict, name, -(1 as i32));
                    prefix = xmlDictLookup_safe((*ctxt).dict, fullname, len)
                }
                /*
                 * make sure there is some storage
                 */
                defaults = xmlHashLookup2_safe((*ctxt).attsDefault, name, prefix) as xmlDefAttrsPtr;
                if defaults.is_null() {
                    defaults =
                        xmlMalloc_safe((::std::mem::size_of::<xmlDefAttrs>() as u64).wrapping_add(
                            ((4 as i32 * 5 as i32) as u64).wrapping_mul(::std::mem::size_of::<
                                *const xmlChar,
                            >(
                            )
                                as u64),
                        )) as xmlDefAttrsPtr;
                    if defaults.is_null() {
                        current_block = 2968889880470072775;
                    } else {
                        (*defaults).nbAttrs = 0 as i32;
                        (*defaults).maxAttrs = 4 as i32;
                        if xmlHashUpdateEntry2(
                            (*ctxt).attsDefault,
                            name,
                            prefix,
                            defaults as *mut (),
                            None,
                        ) < 0 as i32
                        {
                            xmlFree_safe(defaults as *mut ());
                            current_block = 2968889880470072775;
                        } else {
                            current_block = 8704759739624374314;
                        }
                    }
                } else if (*defaults).nbAttrs >= (*defaults).maxAttrs {
                    let mut temp: xmlDefAttrsPtr = 0 as *mut xmlDefAttrs;
                    temp = xmlRealloc_safe(
                        defaults as *mut (),
                        (::std::mem::size_of::<xmlDefAttrs>() as u64).wrapping_add(
                            ((2 as i32 * (*defaults).maxAttrs * 5 as i32) as u64)
                                .wrapping_mul(::std::mem::size_of::<*const xmlChar>() as u64),
                        ),
                    ) as xmlDefAttrsPtr;
                    if temp.is_null() {
                        current_block = 2968889880470072775;
                    } else {
                        defaults = temp;
                        (*defaults).maxAttrs *= 2 as i32;
                        if xmlHashUpdateEntry2(
                            (*ctxt).attsDefault,
                            name,
                            prefix,
                            defaults as *mut (),
                            None,
                        ) < 0 as i32
                        {
                            xmlFree_safe(defaults as *mut ());
                            current_block = 2968889880470072775;
                        } else {
                            current_block = 8704759739624374314;
                        }
                    }
                } else {
                    current_block = 8704759739624374314;
                }
                match current_block {
                    2968889880470072775 => {}
                    _ => {
                        /*
                         * Split the element name into prefix:localname , the string found
                         * are within the DTD and hen not associated to namespace names.
                         */
                        name = xmlSplitQName3(fullattr, &mut len);
                        if name.is_null() {
                            name = xmlDictLookup_safe((*ctxt).dict, fullattr, -(1 as i32));
                            prefix = 0 as *const xmlChar
                        } else {
                            name = xmlDictLookup_safe((*ctxt).dict, name, -(1 as i32));
                            prefix = xmlDictLookup_safe((*ctxt).dict, fullattr, len)
                        }
                        let ref mut fresh3 = *(*defaults)
                            .values
                            .as_mut_ptr()
                            .offset((5 as i32 * (*defaults).nbAttrs) as isize);
                        *fresh3 = name;
                        let ref mut fresh4 = *(*defaults)
                            .values
                            .as_mut_ptr()
                            .offset((5 as i32 * (*defaults).nbAttrs + 1 as i32) as isize);
                        *fresh4 = prefix;
                        /* intern the string and precompute the end */
                        len = xmlStrlen_safe(value);
                        unsafe {
                            value = xmlDictLookup_safe((*ctxt).dict, value, len);
                        }
                        let ref mut fresh5 = *(*defaults)
                            .values
                            .as_mut_ptr()
                            .offset((5 as i32 * (*defaults).nbAttrs + 2 as i32) as isize);
                        *fresh5 = value;
                        let ref mut fresh6 = *(*defaults)
                            .values
                            .as_mut_ptr()
                            .offset((5 as i32 * (*defaults).nbAttrs + 3 as i32) as isize);
                        *fresh6 = value.offset(len as isize);
                        if (*ctxt).external != 0 {
                            let ref mut fresh7 = *(*defaults)
                                .values
                                .as_mut_ptr()
                                .offset((5 as i32 * (*defaults).nbAttrs + 4 as i32) as isize);
                            *fresh7 = b"external\x00" as *const u8 as *const i8 as *mut xmlChar
                        } else {
                            let ref mut fresh8 = *(*defaults)
                                .values
                                .as_mut_ptr()
                                .offset((5 as i32 * (*defaults).nbAttrs + 4 as i32) as isize);
                            *fresh8 = 0 as *const xmlChar
                        }
                        (*defaults).nbAttrs += 1;
                        return;
                    }
                }
            }
            _ => {}
        }
    }
    unsafe { xmlErrMemory(ctxt, 0 as *const i8) };
}
/* *
* xmlAddSpecialAttr:
* @ctxt:  an XML parser context
* @fullname:  the element fullname
* @fullattr:  the attribute fullname
* @type:  the attribute type
*
* Register this attribute type
*/
fn xmlAddSpecialAttr(
    ctxt: xmlParserCtxtPtr,
    fullname: *const xmlChar,
    fullattr: *const xmlChar,
    type_0: i32,
) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).attsSpecial.is_null() {
        (safe_ctxt).attsSpecial = unsafe { xmlHashCreateDict_safe(10, (safe_ctxt).dict) };
        if (safe_ctxt).attsSpecial.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return;
        }
    }
    if !unsafe { xmlHashLookup2_safe(unsafe { (*ctxt).attsSpecial }, fullname, fullattr) }.is_null()
    {
        return;
    }
    unsafe {
        xmlHashAddEntry2_safe(
            unsafe { (*ctxt).attsSpecial },
            fullname,
            fullattr,
            type_0 as ptrdiff_t as *mut (),
        )
    };
}
/* *
* xmlCleanSpecialAttrCallback:
*
* Removes CDATA attributes from the special attribute table
*/
extern "C" fn xmlCleanSpecialAttrCallback(
    mut payload: *mut (),
    mut data: *mut (),
    mut fullname: *const xmlChar,
    mut fullattr: *const xmlChar,
    mut unused: *const xmlChar,
) {
    let mut ctxt: xmlParserCtxtPtr = data as xmlParserCtxtPtr;
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if payload as ptrdiff_t == XML_ATTRIBUTE_CDATA as i32 as i64 {
        unsafe { xmlHashRemoveEntry2_safe((safe_ctxt).attsSpecial, fullname, fullattr, None) };
    };
}
/* *
* xmlCleanSpecialAttr:
* @ctxt:  an XML parser context
*
* Trim the list of attributes defined to remove all those of type
* CDATA as they are not special. This call should be done when finishing
* to parse the DTD and before starting to parse the document root.
*/
fn xmlCleanSpecialAttr(mut ctxt: xmlParserCtxtPtr) {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).attsSpecial.is_null() {
        return;
    }
    unsafe {
        xmlHashScanFull_safe(
            (safe_ctxt).attsSpecial,
            Some(
                xmlCleanSpecialAttrCallback
                    as extern "C" fn(
                        _: *mut (),
                        _: *mut (),
                        _: *const xmlChar,
                        _: *const xmlChar,
                        _: *const xmlChar,
                    ) -> (),
            ),
            ctxt as *mut (),
        );
        if xmlHashSize_safe((safe_ctxt).attsSpecial) == 0 as i32 {
            xmlHashFree_safe((safe_ctxt).attsSpecial, None);
            (safe_ctxt).attsSpecial = 0 as xmlHashTablePtr;
        };
    }
}
/* *
* xmlCheckLanguageID:
* @lang:  pointer to the string value
*
* Checks that the value conforms to the LanguageID production:
*
* NOTE: this is somewhat deprecated, those productions were removed from
*       the XML Second edition.
*
* [33] LanguageID ::= Langcode ('-' Subcode)*
* [34] Langcode ::= ISO639Code |  IanaCode |  UserCode
* [35] ISO639Code ::= ([a-z] | [A-Z]) ([a-z] | [A-Z])
* [36] IanaCode ::= ('i' | 'I') '-' ([a-z] | [A-Z])+
* [37] UserCode ::= ('x' | 'X') '-' ([a-z] | [A-Z])+
* [38] Subcode ::= ([a-z] | [A-Z])+
*
* The current REC reference the successors of RFC 1766, currently 5646
*
* http://www.rfc-editor.org/rfc/rfc5646.txt
* langtag       = language
*                 ["-" script]
*                 ["-" region]
*                 *("-" variant)
*                 *("-" extension)
*                 ["-" privateuse]
* language      = 2*3ALPHA            ; shortest ISO 639 code
*                 ["-" extlang]       ; sometimes followed by
*                                     ; extended language subtags
*               / 4ALPHA              ; or reserved for future use
*               / 5*8ALPHA            ; or registered language subtag
*
* extlang       = 3ALPHA              ; selected ISO 639 codes
*                 *2("-" 3ALPHA)      ; permanently reserved
*
* script        = 4ALPHA              ; ISO 15924 code
*
* region        = 2ALPHA              ; ISO 3166-1 code
*               / 3DIGIT              ; UN M.49 code
*
* variant       = 5*8alphanum         ; registered variants
*               / (DIGIT 3alphanum)
*
* extension     = singleton 1*("-" (2*8alphanum))
*
*                                     ; Single alphanumerics
*                                     ; "x" reserved for private use
* singleton     = DIGIT               ; 0 - 9
*               / %x41-57             ; A - W
*               / %x59-5A             ; Y - Z
*               / %x61-77             ; a - w
*               / %x79-7A             ; y - z
*
* it sounds right to still allow Irregular i-xxx IANA and user codes too
* The parser below doesn't try to cope with extension or privateuse
* that could be added but that's not interoperable anyway
*
* Returns 1 if correct 0 otherwise
**/

pub fn xmlCheckLanguageID(lang: *const xmlChar) -> i32 {
    let mut current_block: u64;
    let mut cur: *const xmlChar = lang;
    let mut nxt: *const xmlChar = 0 as *const xmlChar;
    if cur.is_null() {
        return 0;
    }
    if unsafe {
        *cur.offset(0) as i32 == 'i' as i32 && *cur.offset(1) as i32 == '-' as i32
            || *cur.offset(0) as i32 == 'I' as i32 && *cur.offset(1) as i32 == '-' as i32
            || *cur.offset(0) as i32 == 'x' as i32 && *cur.offset(1) as i32 == '-' as i32
            || *cur.offset(0) as i32 == 'X' as i32 && *cur.offset(1) as i32 == '-' as i32
    } {
        /*
         * Still allow IANA code and user code which were coming
         * from the previous version of the XML-1.0 specification
         * it's deprecated but we should not fail
         */
        cur = unsafe { cur.offset(2) };
        while unsafe {
            *cur.offset(0) as i32 >= 'A' as i32 && *cur.offset(0) as i32 <= 'Z' as i32
                || *cur.offset(0) as i32 >= 'a' as i32 && *cur.offset(0) as i32 <= 'z' as i32
        } {
            cur = unsafe { cur.offset(1) };
        }
        return unsafe { (*cur.offset(0) as i32 == 0 as i32) as i32 };
    }
    nxt = cur;
    while unsafe {
        *nxt.offset(0) as i32 >= 'A' as i32 && *nxt.offset(0) as i32 <= 'Z' as i32
            || *nxt.offset(0) as i32 >= 'a' as i32 && *nxt.offset(0) as i32 <= 'z' as i32
    } {
        nxt = unsafe { nxt.offset(1) };
    }
    if unsafe { nxt.offset_from(cur) as i64 >= 4 } {
        /*
         * Reserved
         */
        if unsafe { nxt.offset_from(cur) as i64 > 8 || *nxt.offset(0) as i32 != 0 as i32 } {
            return 0;
        }
        return 1;
    }
    if unsafe { (nxt.offset_from(cur) as i64) < 2 } {
        return 0;
    }
    /* we got an ISO 639 code */
    if unsafe { *nxt.offset(0 as i32 as isize) as i32 == 0 } {
        return 1;
    }
    if unsafe { *nxt.offset(0 as i32 as isize) as i32 != '-' as i32 } {
        return 0;
    }
    nxt = unsafe { nxt.offset(1) };
    cur = nxt;
    /* now we can have extlang or script or region or variant */
    if unsafe { *nxt.offset(0) as i32 >= '0' as i32 && *nxt.offset(0) as i32 <= '9' as i32 } {
        current_block = 13163178004963364532;
    } else {
        while unsafe {
            *nxt.offset(0 as i32 as isize) as i32 >= 'A' as i32
                && *nxt.offset(0 as i32 as isize) as i32 <= 'Z' as i32
                || *nxt.offset(0 as i32 as isize) as i32 >= 'a' as i32
                    && *nxt.offset(0 as i32 as isize) as i32 <= 'z' as i32
        } {
            nxt = unsafe { nxt.offset(1) };
        }
        if unsafe { nxt.offset_from(cur) as i64 == 4 as i32 as i64 } {
            current_block = 14921549473310263854;
        } else if unsafe { nxt.offset_from(cur) as i64 == 2 as i32 as i64 } {
            current_block = 15970415187932728765;
        } else if unsafe {
            nxt.offset_from(cur) as i64 >= 5 as i32 as i64
                && nxt.offset_from(cur) as i64 <= 8 as i32 as i64
        } {
            current_block = 6166658882887268861;
        } else {
            if unsafe { nxt.offset_from(cur) as i64 != 3 as i32 as i64 } {
                return 0 as i32;
            }
            /* we parsed an extlang */
            if unsafe { *nxt.offset(0 as i32 as isize) as i32 == 0 as i32 } {
                return 1 as i32;
            }
            if unsafe { *nxt.offset(0 as i32 as isize) as i32 != '-' as i32 } {
                return 0 as i32;
            }
            nxt = unsafe { nxt.offset(1) };
            cur = nxt;
            /* now we can have script or region or variant */
            if unsafe {
                *nxt.offset(0 as i32 as isize) as i32 >= '0' as i32
                    && *nxt.offset(0 as i32 as isize) as i32 <= '9' as i32
            } {
                current_block = 13163178004963364532;
            } else {
                while unsafe {
                    *nxt.offset(0 as i32 as isize) as i32 >= 'A' as i32
                        && *nxt.offset(0 as i32 as isize) as i32 <= 'Z' as i32
                        || *nxt.offset(0 as i32 as isize) as i32 >= 'a' as i32
                            && *nxt.offset(0 as i32 as isize) as i32 <= 'z' as i32
                } {
                    nxt = unsafe { nxt.offset(1) };
                }
                if unsafe { nxt.offset_from(cur) as i64 == 2 as i32 as i64 } {
                    current_block = 15970415187932728765;
                } else if unsafe {
                    nxt.offset_from(cur) as i64 >= 5 as i32 as i64
                        && nxt.offset_from(cur) as i64 <= 8 as i32 as i64
                } {
                    current_block = 6166658882887268861;
                } else {
                    if unsafe { nxt.offset_from(cur) as i64 != 4 as i32 as i64 } {
                        return 0 as i32;
                    }
                    current_block = 14921549473310263854;
                }
            }
        }
        match current_block {
            15970415187932728765 => {}
            6166658882887268861 => {}
            13163178004963364532 => {}
            _ =>
            /* we parsed a script */
            {
                if unsafe { *nxt.offset(0 as i32 as isize) as i32 == 0 as i32 } {
                    return 1;
                }
                if unsafe { *nxt.offset(0 as i32 as isize) as i32 != '-' as i32 } {
                    return 0;
                }
                nxt = unsafe { nxt.offset(1) };
                cur = nxt;
                /* now we can have region or variant */
                if unsafe {
                    *nxt.offset(0 as i32 as isize) as i32 >= '0' as i32
                        && *nxt.offset(0 as i32 as isize) as i32 <= '9' as i32
                } {
                    current_block = 13163178004963364532;
                } else {
                    while unsafe {
                        *nxt.offset(0 as i32 as isize) as i32 >= 'A' as i32
                            && *nxt.offset(0 as i32 as isize) as i32 <= 'Z' as i32
                            || *nxt.offset(0 as i32 as isize) as i32 >= 'a' as i32
                                && *nxt.offset(0 as i32 as isize) as i32 <= 'z' as i32
                    } {
                        nxt = unsafe { nxt.offset(1) };
                    }
                    if unsafe {
                        nxt.offset_from(cur) as i64 >= 5 as i32 as i64
                            && nxt.offset_from(cur) as i64 <= 8 as i32 as i64
                    } {
                        current_block = 6166658882887268861;
                    } else {
                        if unsafe { nxt.offset_from(cur) as i64 != 2 as i32 as i64 } {
                            return 0 as i32;
                        }
                        current_block = 15970415187932728765;
                    }
                }
            }
        }
    }
    match current_block {
        13163178004963364532 => {
            if unsafe {
                *nxt.offset(1 as i32 as isize) as i32 >= '0' as i32
                    && *nxt.offset(1 as i32 as isize) as i32 <= '9' as i32
                    && (*nxt.offset(2 as i32 as isize) as i32 >= '0' as i32
                        && *nxt.offset(2 as i32 as isize) as i32 <= '9' as i32)
            } {
                nxt = unsafe { nxt.offset(3 as i32 as isize) }
            } else {
                return 0 as i32;
            }
            current_block = 15970415187932728765;
        }
        _ => {}
    }
    match current_block {
        15970415187932728765 =>
        /* we parsed a region */
        {
            if unsafe { *nxt.offset(0 as i32 as isize) as i32 == 0 as i32 } {
                return 1 as i32;
            }
            if unsafe { *nxt.offset(0 as i32 as isize) as i32 != '-' as i32 } {
                return 0 as i32;
            }
            nxt = unsafe { nxt.offset(1) };
            cur = nxt;
            /* now we can just have a variant */
            while unsafe {
                *nxt.offset(0 as i32 as isize) as i32 >= 'A' as i32
                    && *nxt.offset(0 as i32 as isize) as i32 <= 'Z' as i32
                    || *nxt.offset(0 as i32 as isize) as i32 >= 'a' as i32
                        && *nxt.offset(0 as i32 as isize) as i32 <= 'z' as i32
            } {
                nxt = unsafe { nxt.offset(1) };
            }
            if unsafe {
                (nxt.offset_from(cur) as i64) < 5 as i32 as i64
                    || nxt.offset_from(cur) as i64 > 8 as i32 as i64
            } {
                return 0 as i32;
            }
        }
        _ => {}
    }
    /* we parsed a variant */
    if unsafe { *nxt.offset(0 as i32 as isize) as i32 == 0 as i32 } {
        return 1;
    }
    if unsafe { *nxt.offset(0 as i32 as isize) as i32 != '-' as i32 } {
        return 0;
    }
    /* extensions and private use subtags not checked */
    return 1;
}
/* *
* nsPush:
* @ctxt:  an XML parser context
* @prefix:  the namespace prefix or NULL
* @URL:  the namespace name
*
* Pushes a new parser namespace on top of the ns stack
*
* Returns -1 in case of error, -2 if the namespace should be discarded
*	   and the index in the stack otherwise.
*/
#[cfg(HAVE_parser_SAX2)]
fn nsPush(mut ctxt: xmlParserCtxtPtr, mut prefix: *const xmlChar, mut URL: *const xmlChar) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).options & XML_PARSE_NSCLEAN as i32 != 0 {
        let mut i: i32 = 0;
        i = (safe_ctxt).nsNr - 2 as i32;
        while i >= 0 as i32 {
            if unsafe { *(*ctxt).nsTab.offset(i as isize) == prefix } {
                /* in scope */
                if unsafe { *(*ctxt).nsTab.offset((i + 1 as i32) as isize) == URL } {
                    return -(2 as i32);
                }
                break;
            } else {
                i -= 2 as i32
            }
        }
    }
    if (safe_ctxt).nsMax == 0 as i32 || (safe_ctxt).nsTab.is_null() {
        (safe_ctxt).nsMax = 10 as i32;
        (safe_ctxt).nsNr = 0 as i32;
        (safe_ctxt).nsTab = unsafe {
            xmlMalloc_safe(
                ((safe_ctxt).nsMax as u64)
                    .wrapping_mul(::std::mem::size_of::<*mut xmlChar>() as u64),
            ) as *mut *const xmlChar
        };
        if (safe_ctxt).nsTab.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            (safe_ctxt).nsMax = 0 as i32;
            return -(1 as i32);
        }
    } else if (safe_ctxt).nsNr >= (safe_ctxt).nsMax {
        let mut tmp: *mut *const xmlChar = 0 as *mut *const xmlChar;
        (safe_ctxt).nsMax *= 2 as i32;
        tmp = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).nsTab as *mut i8 as *mut (),
                ((safe_ctxt).nsMax as u64)
                    .wrapping_mul(::std::mem::size_of::<*const xmlChar>() as u64),
            ) as *mut *const xmlChar
        };
        if tmp.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            (safe_ctxt).nsMax /= 2 as i32;
            return -(1 as i32);
        }
        (safe_ctxt).nsTab = tmp
    }
    let fresh9 = (safe_ctxt).nsNr;
    (safe_ctxt).nsNr = (safe_ctxt).nsNr + 1;
    unsafe {
        let ref mut fresh10 = *(*ctxt).nsTab.offset(fresh9 as isize);
        *fresh10 = prefix;
        let fresh11 = (safe_ctxt).nsNr;
        (safe_ctxt).nsNr = (safe_ctxt).nsNr + 1;
        let ref mut fresh12 = *(*ctxt).nsTab.offset(fresh11 as isize);
        *fresh12 = URL;
    };
    return (safe_ctxt).nsNr;
}
/* *
* nsPop:
* @ctxt: an XML parser context
* @nr:  the number to pop
*
* Pops the top @nr parser prefix/namespace from the ns stack
*
* Returns the number of namespaces removed
*/
#[cfg(HAVE_parser_SAX2)]
fn nsPop(mut ctxt: xmlParserCtxtPtr, mut nr: i32) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut i: i32 = 0; /* allow for 10 attrs by default */
    if (safe_ctxt).nsTab.is_null() {
        return 0;
    }
    if (safe_ctxt).nsNr < nr {
        unsafe {
            (*__xmlGenericError()).expect("non-null function pointer")(
                *__xmlGenericErrorContext(),
                b"Pbm popping %d NS\n\x00" as *const u8 as *const i8,
                nr,
            );
        }
        nr = (safe_ctxt).nsNr
    }
    if (safe_ctxt).nsNr <= 0 {
        return 0;
    }
    i = 0;
    while i < nr {
        (safe_ctxt).nsNr -= 1;
        unsafe {
            let ref mut fresh13 = *(*ctxt).nsTab.offset((safe_ctxt).nsNr as isize);
            *fresh13 = 0 as *const xmlChar;
        }
        i += 1
    }
    return nr;
}
fn xmlCtxtGrowAttrs(mut ctxt: xmlParserCtxtPtr, mut nr: i32) -> i32 {
    let mut safe_ctxt = unsafe { &mut *ctxt };
    let mut current_block: u64;
    let mut atts: *mut *const xmlChar = 0 as *mut *const xmlChar;
    let mut attallocs: *mut i32 = 0 as *mut i32;
    let mut maxatts: i32 = 0;
    if (safe_ctxt).atts.is_null() {
        maxatts = 55 as i32;
        atts = unsafe {
            xmlMalloc_safe(
                (maxatts as u64).wrapping_mul(::std::mem::size_of::<*mut xmlChar>() as u64),
            ) as *mut *const xmlChar
        };
        if atts.is_null() {
            current_block = 1220566974040888119;
        } else {
            (safe_ctxt).atts = atts;
            attallocs = unsafe {
                xmlMalloc_safe(
                    ((maxatts / 5 as i32) as u64).wrapping_mul(::std::mem::size_of::<i32>() as u64),
                ) as *mut i32
            };
            if attallocs.is_null() {
                current_block = 1220566974040888119;
            } else {
                (safe_ctxt).attallocs = attallocs;
                (safe_ctxt).maxatts = maxatts;
                current_block = 13242334135786603907;
            }
        }
    } else if nr + 5 as i32 > (safe_ctxt).maxatts {
        maxatts = (nr + 5 as i32) * 2 as i32;
        atts = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).atts as *mut (),
                (maxatts as u64).wrapping_mul(::std::mem::size_of::<*const xmlChar>() as u64),
            ) as *mut *const xmlChar
        };
        if atts.is_null() {
            current_block = 1220566974040888119;
        } else {
            (safe_ctxt).atts = atts;
            attallocs = unsafe {
                xmlRealloc_safe(
                    (safe_ctxt).attallocs as *mut (),
                    ((maxatts / 5 as i32) as u64).wrapping_mul(::std::mem::size_of::<i32>() as u64),
                ) as *mut i32
            };
            if attallocs.is_null() {
                current_block = 1220566974040888119;
            } else {
                (safe_ctxt).attallocs = attallocs;
                (safe_ctxt).maxatts = maxatts;
                current_block = 13242334135786603907;
            }
        }
    } else {
        current_block = 13242334135786603907;
    }
    match current_block {
        13242334135786603907 => return (safe_ctxt).maxatts,
        _ => {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return -1;
        }
    };
}
/* *
* inputPush:
* @ctxt:  an XML parser context
* @value:  the parser input
*
* Pushes a new parser input on top of the input stack
*
* Returns -1 in case of error, the index in the stack otherwise
*/

pub fn inputPush_parser(ctxt: xmlParserCtxtPtr, mut value: xmlParserInputPtr) -> i32 {
    if ctxt.is_null() || value.is_null() {
        return -(1 as i32);
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).inputNr >= (safe_ctxt).inputMax {
        (safe_ctxt).inputMax *= 2 as i32;
        (safe_ctxt).inputTab = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).inputTab as *mut (),
                ((safe_ctxt).inputMax as u64)
                    .wrapping_mul(::std::mem::size_of::<xmlParserInputPtr>() as u64),
            ) as *mut xmlParserInputPtr
        };
        if (safe_ctxt).inputTab.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            unsafe {
                xmlFreeInputStream_safe(value);
            }
            (safe_ctxt).inputMax /= 2 as i32;
            value = 0 as xmlParserInputPtr;
            return -1;
        }
    }
    unsafe {
        let ref mut fresh14 = *(*ctxt).inputTab.offset((safe_ctxt).inputNr as isize);
        *fresh14 = value;
    }
    (safe_ctxt).input = value;
    let fresh15 = (safe_ctxt).inputNr;
    (safe_ctxt).inputNr = (safe_ctxt).inputNr + 1;
    return fresh15;
}
/* *
* inputPop:
* @ctxt: an XML parser context
*
* Pops the top parser input from the input stack
*
* Returns the input just removed
*/

pub fn inputPop_parser(mut ctxt: xmlParserCtxtPtr) -> xmlParserInputPtr {
    let mut ret: xmlParserInputPtr = 0 as *mut xmlParserInput;
    if ctxt.is_null() {
        return 0 as xmlParserInputPtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).inputNr <= 0 {
        return 0 as xmlParserInputPtr;
    }
    (safe_ctxt).inputNr -= 1;
    if (safe_ctxt).inputNr > 0 {
        (safe_ctxt).input = unsafe { *(*ctxt).inputTab.offset(((safe_ctxt).inputNr - 1) as isize) };
    } else {
        (safe_ctxt).input = 0 as xmlParserInputPtr
    }
    unsafe {
        ret = *(*ctxt).inputTab.offset((safe_ctxt).inputNr as isize);
        let ref mut fresh16 = *(*ctxt).inputTab.offset((safe_ctxt).inputNr as isize);
        *fresh16 = 0 as xmlParserInputPtr;
    }
    return ret;
}
/* *
* nodePush:
* @ctxt:  an XML parser context
* @value:  the element node
*
* Pushes a new element node on top of the node stack
*
* Returns -1 in case of error, the index in the stack otherwise
*/

pub fn nodePush(ctxt: xmlParserCtxtPtr, value: xmlNodePtr) -> i32 {
    if ctxt.is_null() {
        return 0;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).nodeNr >= (safe_ctxt).nodeMax {
        let mut tmp: *mut xmlNodePtr = 0 as *mut xmlNodePtr;
        tmp = unsafe {
            xmlRealloc_safe(
                (safe_ctxt).nodeTab as *mut (),
                (((safe_ctxt).nodeMax * 2 as i32) as u64)
                    .wrapping_mul(::std::mem::size_of::<xmlNodePtr>() as u64),
            ) as *mut xmlNodePtr
        };
        if tmp.is_null() {
            unsafe {
                xmlErrMemory(ctxt, 0 as *const i8);
            }
            return -(1 as i32);
        }
        (safe_ctxt).nodeTab = tmp;
        (safe_ctxt).nodeMax *= 2 as i32
    }
    if (safe_ctxt).nodeNr as u32 > unsafe { xmlParserMaxDepth }
        && (safe_ctxt).options & XML_PARSE_HUGE as i32 == 0 as i32
    {
        unsafe {
            xmlFatalErrMsgInt(
                ctxt,
                XML_ERR_INTERNAL_ERROR,
                b"Excessive depth in document: %d use XML_PARSE_HUGE option\n\x00" as *const u8
                    as *const i8,
                xmlParserMaxDepth as i32,
            );
            xmlHaltParser(ctxt);
        }
        return -(1 as i32);
    }
    unsafe {
        let ref mut fresh17 = *(*ctxt).nodeTab.offset((safe_ctxt).nodeNr as isize);
        *fresh17 = value;
    }
    (safe_ctxt).node = value;
    let fresh18 = (safe_ctxt).nodeNr;
    (safe_ctxt).nodeNr = (safe_ctxt).nodeNr + 1;
    return fresh18;
}
/* *
* nodePop:
* @ctxt: an XML parser context
*
* Pops the top element node from the node stack
*
* Returns the node just removed
*/

pub fn nodePop_parser(mut ctxt: xmlParserCtxtPtr) -> xmlNodePtr {
    let mut ret: xmlNodePtr = 0 as *mut xmlNode;
    if ctxt.is_null() {
        return 0 as xmlNodePtr;
    }
    let mut safe_ctxt = unsafe { &mut *ctxt };
    if (safe_ctxt).nodeNr <= 0 as i32 {
        return 0 as xmlNodePtr;
    }
    (safe_ctxt).nodeNr -= 1;
    if (safe_ctxt).nodeNr > 0 as i32 {
        (safe_ctxt).node = unsafe {
            *(*ctxt)
                .nodeTab
                .offset(((safe_ctxt).nodeNr - 1 as i32) as isize)
        };
    } else {
        (safe_ctxt).node = 0 as xmlNodePtr
    }
    unsafe {
        ret = *(*ctxt).nodeTab.offset((safe_ctxt).nodeNr as isize);
        let ref mut fresh19 = *(*ctxt).nodeTab.offset((safe_ctxt).nodeNr as isize);
        *fresh19 = 0 as xmlNodePtr;
    }
    return ret;
}
