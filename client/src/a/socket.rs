use crate::a::GameState;
use byteorder::{ByteOrder, LittleEndian};
use js_sys::{ArrayBuffer, Uint8Array};

extern crate console_error_panic_hook;
use std::panic;

use js_sys::Reflect;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    ErrorEvent, MessageEvent, RtcDataChannel, RtcDataChannelEvent, RtcDataChannelType,
    RtcIceCandidateInit, RtcPeerConnection, RtcPeerConnectionIceEvent, RtcSdpType,
    RtcSessionDescriptionInit, RtcSignalingState, WebSocket,
};

use std::collections::HashMap;

static mut SOCKET_BUFFER_1: Vec<SocketMessage> = Vec::new();
static mut SOCKET_BUFFER_2: Vec<SocketMessage> = Vec::new();
static mut CURRENT_BUFFER: u8 = 0;

static mut PEERS: Option<HashMap<u8, RtcPeerConnection>> = None;
static mut DATA_CHANNELS: Option<HashMap<u8, RtcDataChannel>> = None;
static mut PEER_ID: Option<u8> = Some(255u8);
static mut SOCKET_EVENTS_1: Vec<ArrayBuffer> = Vec::new();
static mut SOCKET_EVENTS_2: Vec<ArrayBuffer> = Vec::new();
static mut SOCKET_EVENTS_SWITCH: u8 = 0u8;

macro_rules! console_log {
    ($($t:tt)*) => (nodejs_helper::console::log(&format_args!($($t)*).to_string()))
}
macro_rules! console_warn {
    ($($t:tt)*) => (nodejs_helper::console::log(&format_args!($($t)*).to_string()))
}
macro_rules! err_to_console {
    ($($t:tt)*) => (match $($t)* {
        Ok(can) => can,
        Err(err) => {
            nodejs_helper::console::log(&format_args!("{:?}", err).to_string());
            panic!("!!");
        }
    })
}

#[wasm_bindgen]
pub struct Socketry {
    ws: WebSocket,
}

#[wasm_bindgen]
impl Socketry {
    pub fn new() -> Socketry {
        use std::panic;
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        unsafe {
            PEERS = Some(HashMap::new());
            DATA_CHANNELS = Some(HashMap::new());
        }
        console_log!("Start sockets");
        // Connect to an echo server
        let ws = WebSocket::new_with_str("ws://127.0.0.1:13125", "rust-websocket")
            .expect("Could not establish socket connection");
        // For small binary messages, like CBOR, Arraybuffer is more efficient than Blob handling
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);
        // create callback
        //let ws_clone = ws.clone();
        let onmessage_callback = Closure::wrap(Box::new(move |e: MessageEvent| {
            // Handle difference Text/Binary,...
            if let Ok(abuf) = e.data().dyn_into::<js_sys::ArrayBuffer>() {
                unsafe {
                    match SOCKET_EVENTS_SWITCH {
                        0 => SOCKET_EVENTS_1.push(abuf),
                        1 => SOCKET_EVENTS_2.push(abuf),
                        _ => panic!("Unreachable"),
                    }
                }
            } else {
                console_log!("message event, received Unknown: {:?}", e.data());
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        // set message event handler on WebSocket
        ws.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        // forget the callback to keep it alive
        onmessage_callback.forget();
        let onerror_callback = Closure::wrap(Box::new(move |e: ErrorEvent| {
            console_log!("error event: {:?}", e);
        }) as Box<dyn FnMut(ErrorEvent)>);
        ws.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        let cloned_ws = ws.clone();
        let onopen_callback = Closure::wrap(Box::new(move |_| {
            console_log!("socket opened");
            // send off binary message
            match cloned_ws.send_with_u8_array(&vec![0]) {
                Ok(_) => console_log!("binary message successfully sent"),
                Err(err) => console_log!("error sending message: {:?}", err),
            }
        }) as Box<dyn FnMut(JsValue)>);
        ws.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        Socketry { ws }
    }

    pub fn has_socket_events() -> bool {
        unsafe {
            return match SOCKET_EVENTS_SWITCH {
                0 => SOCKET_EVENTS_1.len(),
                1 => SOCKET_EVENTS_2.len(),
                _ => panic!("unreachable"),
            } != 0usize;
        }
    }

    pub async fn handle_socket_events(self) -> Socketry {
        unsafe {
            SOCKET_EVENTS_SWITCH = match SOCKET_EVENTS_SWITCH {
                0 => 1,
                1 => 0,
                _ => panic!("unreachable"),
            };
            for abuf in match SOCKET_EVENTS_SWITCH {
                0 => SOCKET_EVENTS_2.drain(..),
                1 => SOCKET_EVENTS_1.drain(..),
                _ => panic!("unreachable"),
            } {
                let array = js_sys::Uint8Array::new(&abuf);
                let len = array.byte_length() as usize;
                console_log!("Arraybuffer received {}bytes: {:?}", len, array.to_vec());
                match array.to_vec()[0] {
                    0 => {
                        let id = array.to_vec()[1];
                        PEER_ID = Some(id);
                        for remote_id in array.to_vec()[2..].iter() {
                            let (offer, peer) = offer_peer(id, *remote_id).await;
                            (&mut PEERS).as_mut().unwrap().insert(*remote_id, peer);
                            self.ws.clone().send_with_u8_array(
                                &[&vec![1u8, id, *remote_id], offer.as_bytes()].concat(),
                            );
                        }
                    }
                    1 => {
                        let remote_id = array.to_vec()[1];
                        let offer = &array.to_vec()[3..];

                        let (answer, peer) =
                            connect_to_peer(String::from_utf8(offer.to_vec()).unwrap(), remote_id)
                                .await;

                        (&mut PEERS).as_mut().unwrap().insert(remote_id, peer);
                        self.ws.clone().send_with_u8_array(
                            &[&vec![2u8, PEER_ID.unwrap(), remote_id], answer.as_bytes()].concat(),
                        );
                    }
                    2 => {
                        let remote_id = array.to_vec()[1];
                        let answer = &array.to_vec()[3..];

                        connect_to_remote(
                            String::from_utf8(answer.to_vec()).unwrap(),
                            (&mut PEERS).as_mut().unwrap().get_mut(&remote_id).unwrap(),
                        )
                        .await;
                    }
                    3 => {
                        let remote_id = array.to_vec()[2];
                        if let RtcSignalingState::Stable = (&mut PEERS)
                            .as_mut()
                            .unwrap()
                            .get_mut(&remote_id)
                            .unwrap()
                            .signaling_state()
                        {
                            let answer = &array.to_vec()[3..];
                            self.ws.clone().send_with_u8_array(
                                &[&vec![4u8, PEER_ID.unwrap(), remote_id], answer].concat(),
                            );
                        } else {
                            match SOCKET_EVENTS_SWITCH {
                                0 => SOCKET_EVENTS_1.push(abuf),
                                1 => SOCKET_EVENTS_2.push(abuf),
                                _ => panic!("unreachable"),
                            }
                        }
                    }
                    4 => {
                        let remote_id = array.to_vec()[1];
                        if let RtcSignalingState::Stable = (&mut PEERS)
                            .as_mut()
                            .unwrap()
                            .get_mut(&remote_id)
                            .unwrap()
                            .signaling_state()
                        {
                            let candidate_full = &array.to_vec()[3..];
                            console_log!(
                                "Candidate: {:?}",
                                String::from_utf8(candidate_full.to_vec()).unwrap()
                            );

                            let i_mid =
                                candidate_full.iter().position(|&x| x == ';' as u8).unwrap();
                            let i_m_line_index =
                                candidate_full.iter().position(|&x| x == ',' as u8).unwrap();
                            let mid = &candidate_full[0..i_mid];
                            let m_line_index = &candidate_full[i_mid + 1..i_m_line_index];
                            let candidate = &candidate_full[i_m_line_index + 1..];
                            connect_to_candidate(
                                String::from_utf8(mid.to_vec()).unwrap(),
                                LittleEndian::read_u16(m_line_index),
                                String::from_utf8(candidate.to_vec()).unwrap(),
                                (&mut PEERS).as_mut().unwrap().get_mut(&remote_id).unwrap(),
                            )
                            .await;
                        } else {
                            match SOCKET_EVENTS_SWITCH {
                                0 => SOCKET_EVENTS_1.push(abuf),
                                1 => SOCKET_EVENTS_2.push(abuf),
                                _ => panic!("unreachable"),
                            }
                        }
                    }
                    255 => {
                        let close_id = array.to_vec()[1];
                        let my_new_id = array.to_vec()[2];

                        console_log!("Close {:?}", PEERS);
                        if let Some(peer) = PEERS.as_mut().unwrap().remove(&close_id) {
                            peer.close();
                        }
                        if let Some(dc) = DATA_CHANNELS.as_mut().unwrap().remove(&close_id) {
                            dc.close();
                        }

                        PEER_ID = Some(my_new_id);
                    }
                    _ => console_warn!("Unrecognized {:?}", array.to_vec()),
                }
            }
        }
        self
    }
}

use crate::a::e::p::PlayerState;
use packed_simd::f32x2;
use std::mem;
use std::vec::IntoIter;

pub fn bytes_f<'a>(fi: f32) -> Vec<u8> {
    let mut fs = [0u8; mem::size_of::<f32>()];
    LittleEndian::write_f32(&mut fs, fi);
    fs.to_vec()
}
pub fn read_f(fs: Vec<u8>) -> f32 {
    LittleEndian::read_f32(&fs)
}
#[derive(Debug)]
pub enum SocketMessage {
    Player(u8, PlayerState, f32x2, f32x2),
    Skip,
}
impl SocketMessage {
    pub fn buffer(&self) -> Vec<Vec<u8>> {
        match self {
            SocketMessage::Player(_, _player_state, xy, vel) => vec![
                unsafe { vec![PEER_ID.unwrap()] },
                vec![1u8, 0u8],
                vec![0u8, 0u8],
                bytes_f(xy.extract(0)),
                bytes_f(xy.extract(1)),
                bytes_f(vel.extract(0)),
                bytes_f(vel.extract(1)),
            ],
            SocketMessage::Skip => vec![],
        }
    }

    pub fn read(iter: &mut IntoIter<u8>) -> Option<SocketMessage> {
        match (iter.next(), iter.next(), iter.next()) {
            (Some(remote_id), Some(b0), Some(b1)) if b0 == 1u8 && b1 == 0u8 => {
                let player = Some(SocketMessage::Player(
                    remote_id,
                    match (iter.next(), iter.next()) {
                        (Some(b2), Some(b3)) if b2 == 0u8 && b3 == 0u8 => PlayerState::Idle,
                        _ => panic!("unknown message"),
                    },
                    f32x2::new(
                        read_f(iter.take(4).collect()),
                        read_f(iter.take(4).collect()),
                    ),
                    f32x2::new(
                        read_f(iter.take(4).collect()),
                        read_f(iter.take(4).collect()),
                    ),
                ));
                player
            }
            _ => None,
        }
    }
}

pub fn read_from_sockets(game: &mut GameState) {
    unsafe {
        CURRENT_BUFFER = match CURRENT_BUFFER {
            0 => 1,
            1 => 0,
            _ => panic!("Unreeeeachable"),
        };
        for msg in match CURRENT_BUFFER {
            0 => SOCKET_BUFFER_2.drain(..),
            1 => SOCKET_BUFFER_1.drain(..),
            _ => panic!("Unreeeeachable"),
        } {
            match msg {
                SocketMessage::Player(remote_id, _, _, _)
                    if remote_id != unsafe { PEER_ID.unwrap() } =>
                {
                    let remote_index = match unsafe { PEER_ID.unwrap() } {
                        player_id if remote_id >= player_id => remote_id - 1,
                        player_id if remote_id < player_id => remote_id,
                        _ => panic!("Either player id is greater, or lesser"),
                    };
                    console_log!("{}", remote_index);
                    game.handle_enemy_player(remote_index, msg)
                }
                SocketMessage::Skip => {}
                _ => { /*Skip fall through*/ }
            }
        }
    }
}
pub fn write_to_sockets(msgs: Vec<SocketMessage>) {
    let msg: Vec<u8> = msgs
        .into_iter()
        .flat_map(|x| x.buffer().into_iter().flat_map(|y| y.into_iter()))
        .collect();
    unsafe {
        for (_, chnl) in DATA_CHANNELS.as_ref().unwrap() {
            //console_log!("Send:{:?}", &msg.clone());
            chnl.send_with_u8_array(&msg.clone());
        }
    }
}

use js_sys::{Object, JSON};
pub async fn offer_peer(id: u8, remote_id: u8) -> (String, RtcPeerConnection) {
    console_log!("Start peer");
    let peer = RtcPeerConnection::new().expect("Could not create peer");
    console_log!("Created peer connection");

    let onchannel_callback = Closure::wrap(Box::new(move |_ev: RtcDataChannelEvent| {
        console_log!("Hello data?");
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    peer.set_ondatachannel(Some(onchannel_callback.as_ref().unchecked_ref()));
    onchannel_callback.forget();
    //
    let dc1 = peer.create_data_channel("data");
    dc1.set_binary_type(RtcDataChannelType::Arraybuffer);
    console_log!("dc1 created: label {:?}", dc1.label());

    let _dc1c = dc1.clone();
    let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
        if let Ok(abuf) = ev.data().dyn_into::<js_sys::ArrayBuffer>() {
            unsafe {
                let mut itr = Uint8Array::new(&abuf).to_vec().into_iter();
                while let Some(msg) = SocketMessage::read(&mut itr) {
                    match CURRENT_BUFFER {
                        0 => SOCKET_BUFFER_1.push(msg),
                        1 => SOCKET_BUFFER_2.push(msg),
                        _ => panic!("Unreeeeachable"),
                    }
                }
            }
        } else if let Ok(txt) = ev.data().dyn_into::<js_sys::JsString>() {
            nodejs_helper::console::log(
                format!("message event, received Text: {:?}", txt).as_str(),
            );
        } else {
            nodejs_helper::console::log(
                format!("message event, received Unknown: {:?}", ev.data()).as_str(),
            );
        }
    }) as Box<dyn FnMut(MessageEvent)>);
    dc1.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
    onmessage_callback.forget();
    let onopen_callback = Closure::wrap(Box::new(move || {
        console_log!("data channel open");
    }) as Box<dyn FnMut()>);
    dc1.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
    onopen_callback.forget();

    console_log!("After open forget");
    //
    let onclose_callback = Closure::wrap(Box::new(move |_ev: RtcDataChannelEvent| {
        console_log!("data channel close");
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    dc1.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
    onclose_callback.forget();

    let onicecandidate_callback1 =
        Closure::wrap(
            Box::new(move |ev: RtcPeerConnectionIceEvent| match ev.candidate() {
                Some(candidate) => {
                    console_log!(
                        "pc1.onicecandidate: {:?}",
                        AsRef::<Object>::as_ref(
                            &JSON::stringify(candidate.as_ref())
                                .expect("Stringify Candidate failed"),
                        )
                        .as_string()
                        .expect("Stringify Unwrap failed"),
                    );
                    unsafe {
                        let mut buf = [0u8; 2];
                        LittleEndian::write_u16(&mut buf, candidate.sdp_m_line_index().unwrap());
                        match SOCKET_EVENTS_SWITCH {
                            0 => SOCKET_EVENTS_1.push(
                                Uint8Array::from(
                                    (&[
                                        &vec![3u8, id, remote_id],
                                        candidate.sdp_mid().unwrap().as_bytes(),
                                        &vec![';' as u8],
                                        &buf,
                                        &vec![',' as u8],
                                        candidate.candidate().as_bytes(),
                                    ]
                                    .concat()) as &[u8],
                                )
                                .buffer(),
                            ),
                            1 => SOCKET_EVENTS_2.push(
                                Uint8Array::from(
                                    (&[
                                        &vec![3u8, id, remote_id],
                                        candidate.sdp_mid().unwrap().as_bytes(),
                                        &vec![';' as u8],
                                        &buf,
                                        &vec![',' as u8],
                                        candidate.candidate().as_bytes(),
                                    ]
                                    .concat()) as &[u8],
                                )
                                .buffer(),
                            ),
                            _ => panic!("Unreeeeachable"),
                        };
                    }
                }
                None => {}
            }) as Box<dyn FnMut(RtcPeerConnectionIceEvent)>,
        );
    peer.set_onicecandidate(Some(onicecandidate_callback1.as_ref().unchecked_ref()));
    onicecandidate_callback1.forget();
    console_log!("After open onice");

    let offer = err_to_console!(JsFuture::from(peer.create_offer()).await);
    console_log!("Offer successful: {:?}", &offer);
    let offer_sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))
        .expect("Could not get sdp from offer")
        .as_string()
        .unwrap();
    console_log!("peer: offer {:?}", offer_sdp);

    let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.sdp(&offer_sdp);
    let sld_promise = peer.set_local_description(&offer_obj);
    JsFuture::from(sld_promise)
        .await
        .expect("Could not set offer as local description");
    console_log!("peer: state get remote {:?}", peer.signaling_state());

    unsafe {
        DATA_CHANNELS.as_mut().unwrap().insert(remote_id, dc1);
    }

    (offer_sdp, peer)
}

pub async fn connect_to_peer(offer_sdp: String, remote_id: u8) -> (String, RtcPeerConnection) {
    let peer = RtcPeerConnection::new().expect("Could not create peer!");
    let ondatachannel_callback = Closure::wrap(Box::new(move |ev: RtcDataChannelEvent| {
        let dc2 = ev.channel();
        console_log!("pc2.ondatachannel!: {:?}", dc2.label());

        let onmessage_callback = Closure::wrap(Box::new(move |ev: MessageEvent| {
            if let Ok(abuf) = ev.data().dyn_into::<js_sys::ArrayBuffer>() {
                unsafe {
                    let mut itr = Uint8Array::new(&abuf).to_vec().into_iter();
                    while let Some(msg) = SocketMessage::read(&mut itr) {
                        match CURRENT_BUFFER {
                            0 => SOCKET_BUFFER_1.push(msg),
                            1 => SOCKET_BUFFER_2.push(msg),
                            _ => panic!("Unreeeeachable"),
                        }
                    }
                }
            } else if let Ok(txt) = ev.data().dyn_into::<js_sys::JsString>() {
                nodejs_helper::console::log(
                    format!("message event, received Text: {:?}", txt).as_str(),
                );
            } else {
                nodejs_helper::console::log(
                    format!("message event, received Unknown: {:?}", ev.data()).as_str(),
                );
            }
        }) as Box<dyn FnMut(MessageEvent)>);
        dc2.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();

        unsafe {
            DATA_CHANNELS.as_mut().unwrap().insert(remote_id, dc2);
        }
    }) as Box<dyn FnMut(RtcDataChannelEvent)>);
    peer.set_ondatachannel(Some(ondatachannel_callback.as_ref().unchecked_ref()));
    ondatachannel_callback.forget();
    let mut offer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Offer);
    offer_obj.sdp(&offer_sdp);
    let srd_promise = peer.set_remote_description(&offer_obj);
    console_log!("{}", offer_sdp.as_str());
    JsFuture::from(srd_promise)
        .await
        .expect("Srd promise fail 1");
    console_log!("pc2: state {:?}", peer.signaling_state());

    let answer = JsFuture::from(peer.create_answer())
        .await
        .expect("Srd promise fail");
    let answer_sdp = Reflect::get(&answer, &JsValue::from_str("sdp"))
        .expect("Srd promise fail")
        .as_string()
        .unwrap();
    console_log!("pc2: answer {:?}", answer_sdp);

    let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.sdp(&answer_sdp);
    let sld_promise = peer.set_local_description(&answer_obj);
    JsFuture::from(sld_promise).await.expect("Srd promise fail");
    console_log!("pc2: state {:?}", peer.signaling_state());

    (answer_sdp, peer)
}

pub async fn connect_to_remote(answer_sdp: String, peer: &mut RtcPeerConnection) {
    let mut answer_obj = RtcSessionDescriptionInit::new(RtcSdpType::Answer);
    answer_obj.sdp(&answer_sdp);
    let sld_promise = peer.set_remote_description(&answer_obj);
    JsFuture::from(sld_promise)
        .await
        .expect("Connect to remote promise fail");
    console_log!("pc2: state {:?}", peer.signaling_state());

    /*
    unsafe {
        (&mut DATA_CHANNELS.clone().unwrap())
            .send_with_str("Ping from pc2.dc!!")
            .unwrap();
    }
    */
}

pub async fn connect_to_candidate(
    mid: String,
    m_line_index: u16,
    candidate_sdp: String,
    peer: &mut RtcPeerConnection,
) {
    console_log!("pc2: state {:?}", peer.signaling_state());
    let mut candidate_obj = RtcIceCandidateInit::new(&candidate_sdp);
    let sld_promise = peer.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
        candidate_obj
            .sdp_mid(Some(&mid))
            .sdp_m_line_index(Some(m_line_index)),
    ));
    JsFuture::from(sld_promise)
        .await
        .expect("Connect to candidate promise fail");
    console_log!("pc2: state {:?}", peer.signaling_state());
}
