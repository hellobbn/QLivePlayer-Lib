use structure::byteorder::{LittleEndian, ReadBytesExt};
// Danmuku for CC, Credit: https://github.com/wbt5/real-url/
use reqwest::Url;

// use tokio::time::sleep;
use serde::Deserialize;
use serde_json::json;
use tokio_tungstenite::connect_async;
use uuid::Uuid;
use std::{time::SystemTime, io::Read};
use std::collections::HashMap;
use std::sync::Arc;
use std::collections::LinkedList;
use std::sync::Mutex;
use rmp_serde;
use rmp;
use futures::{stream::StreamExt, SinkExt};
use tokio::time::sleep;
use flate2::read::ZlibDecoder;
use rmpv;

pub struct CC {
    api1: String,
}

impl CC {
    pub fn new() -> Self {
        CC {
            api1: "https://api.cc.163.com/v1/activitylives/anchor/lives".to_string(),
        }
    }

    fn get_reg(&self) -> Vec<u8> {
        let sid = 6144;
        let cid = 2;

        let update_req_info = json!({
            "22": 640u32,
            "23": 360u32,
            "24": "web",
            "25": "Linux",
            "29": "163_cc",
            "30": "",
            "31": "Mozilla/5.0 (Linux; Android 5.0; SM-G900P Build/LRX21T) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/72.0.3626.121 Mobile Safari/538.36",
        });
        let mac_add = "776ffa9d-b14d-49e0-adec-3b4b94cdc5b9".to_string() + "@web.cc.163.com";
        let device_token = mac_add.clone();

        let n: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
            Ok(t) => t.as_secs(),
            _ => 0,
        };

        let data = json!({
            "web-cc": n * 1000,
            "macAdd": mac_add,
            "device_token": device_token,
            "page_uuid": "1268abe2-ff5c-4920-a775-acec9db304cd",
            "update_req_info": update_req_info,
            "system": "win",
            "memory": 1u32,
            "version": 1u32,
            "webccType": 4253u32
        });
        let mut reg_data : Vec<u8> =  Vec::new();
        let s = structure!("<HHI");
        reg_data = s.pack(sid, cid, 0).unwrap();
        // println!("reg_data before: {:?}", reg_data);

        reg_data.append(&mut self.encode_dict(data));

        // println!("reg_data: {:?}", reg_data);

        reg_data
    }

    fn get_beat(&self) -> Vec<u8> {
        let sid = 6144;
        let cid = 5;
        let data = json!({});
        let mut beat_data = structure!("<HHI").pack(sid, cid, 0).unwrap();
        beat_data.append(&mut self.encode_dict(data));
        beat_data
    }

    fn get_join(&self, data_cid: u32, data_gametype: u32, data_roomid: u32) -> Vec<u8> {
        let sid = 512;
        let cid = 1;
        let data = json!({
            "cid": data_cid,
            "gametype": data_gametype,
            "roomId": data_roomid
        });

        let mut join_data = structure!("<HHI").pack(sid, cid, 0).unwrap();
        join_data.append(&mut self.encode_dict(data));

        join_data
    }

    fn encode_dict(&self, d: serde_json::Value) -> Vec<u8> {
        let x = rmp_serde::to_vec(&d).unwrap();
        x
    }

    pub async fn get_ws_info(&self, url: &str) -> Result<(String, Vec<Vec<u8>>), Box<dyn std::error::Error>> {
        let mut reg_datas = Vec::new();
        let rid = Url::parse(url)?.path_segments().ok_or("rid parse error 1")?.last().ok_or("rid parse error 2")?.to_string();
        let client = reqwest::Client::new();
        let mut param1 = Vec::new();
        param1.push(("anchor_ccid", &rid));
        let resp =
            client.get(&self.api1).header("User-Agent", crate::utils::gen_ua()).query(&param1).send().await?.json::<serde_json::Value>().await?;
        println!("resp: {}", resp.to_string());
        let j = resp.pointer(&format!("/data/{}", rid)).ok_or("Cannot parse json 1")?;
        let channel_id = j.pointer("/channel_id").ok_or("Cannot parse json 2")?.to_string();
        let room_id = j.pointer("/room_id").ok_or("Cannot parse json 3")?.to_string();
        let gametype = j.pointer("/gametype").ok_or("Cannot parse json 4")?.to_string();
        let reg_data = self.get_reg();
        reg_datas.push(reg_data);

        reg_datas.push(self.get_beat());
        reg_datas.push(self.get_join(channel_id.parse::<u32>().unwrap(),
                                gametype.parse::<u32>().unwrap(),
                                room_id.parse::<u32>().unwrap()));

        println!("reg_data: {:?}", reg_datas);
        Ok(("wss://weblink.cc.163.com/".to_string(), reg_datas))
    }

    fn decode_msg(&self, data: &mut Vec<u8>) {
        let mut dat_slice = data.as_slice();
        // println!("Original: {:?}", dat_slice);
        // Offset 0
        let ccsid = dat_slice.read_u16::<LittleEndian>().unwrap();
        // Offset 2
        let cccid = dat_slice.read_u16::<LittleEndian>().unwrap();
        if (ccsid == 515 && cccid == 32785) {
            let mut o = Vec::new();
            let t = dat_slice.read_u32::<LittleEndian>().unwrap();
            println!("ccsid: {} - cccid: {}", ccsid, cccid);

            // Offset 4
            if t != 0u32 {
                // Offset 8
                let s = dat_slice.read_u32::<LittleEndian>().unwrap();
                // Offset 12
                let u = &(dat_slice.clone());
                if u.len() as u32 == s {
                    println!("decompress");
                    let mut de = ZlibDecoder::new(*u);
                    de.read_to_end(&mut o).ok();
                }
            } else {
                o = (dat_slice.clone()).to_vec();
            }
            println!("o = {:?}", o);
            let i : rmpv::Value = rmp_serde::decode::from_slice(o.as_slice()).unwrap();
            let json = serde_json::to_string_pretty(&i).unwrap();

            println!("{}", json);
        }
    }

    pub async fn run(&self, url: &str, dm_fifo: Arc<Mutex<LinkedList<HashMap<String, String>>>>)
        -> Result<(), Box<dyn std::error::Error>> {
        let (ws, reg_data) = self.get_ws_info(url).await?;
        let (ws_stream, _) = connect_async(&ws).await?;
        let (mut ws_write, mut ws_read) = ws_stream.split();
        println!("==> Send reg_data[0]");
        ws_write.send(tokio_tungstenite::tungstenite::Message::Binary(reg_data[0].to_vec())).await?;
        println!("==> Send reg_data[1]");
        ws_write.send(tokio_tungstenite::tungstenite::Message::Binary(reg_data[1].to_vec())).await?;
        println!("==> Send reg_data[2]");
        ws_write.send(tokio_tungstenite::tungstenite::Message::Binary(reg_data[2].to_vec())).await?;
        let hb = self.get_beat();
        tokio::spawn(async move {
            loop {
                println!("==> Send heartbeat");
                sleep(tokio::time::Duration::from_secs(30)).await;
                let hb1 = hb.clone();
                match ws_write.send(tokio_tungstenite::tungstenite::Message::Binary(hb1)).await {
                    Ok(_) => {}
                    _ => {
                        println!("Failed to send heartbeat!");
                    }
                };
            }
        });

        while let Some(m) = ws_read.next().await {
            println!("==> Received message");
            match m {
                Ok(it) => {
                    // let mut dm = self.decode_msg(it.into_data().as_mut())?;
                    // if let Ok(mut df) = dm_fifo.lock() {
                    //     for d in dm.drain(..) {
                    //         df.push_back(d);
                    //     }
                    // }
                    self.decode_msg(it.into_data().as_mut());
                }
                Err(e) => {
                    println!("read ws error: {:?}", e);
                }
            }
        }

        println!("wss closed!");
        Ok(())
    }
}
