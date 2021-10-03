use regex::Regex;
use std::collections::HashMap;

pub struct Youtube {}

impl Youtube {
    pub fn new() -> Self {
        Youtube {}
    }

    pub async fn get_live(&self, room_url: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut cid = "".to_owned();
        let mut vurl = if room_url.contains("youtube.com/channel/") {
            let re = Regex::new(r"youtube.com/channel/([^/?]+)").unwrap();
            cid.push_str(re.captures(room_url).ok_or("get_live err 1")?[1].to_string().as_str());
            format!("https://www.youtube.com/channel/{}/live", &cid)
        } else {
            room_url.to_string()
        };
        let mut ret = HashMap::new();
        let mut dash_urls = "".to_owned();
        for _ in 0u8..=1u8 {
            let resp = client
                .get(&vurl)
                .header("User-Agent", crate::utils::gen_ua())
                .header("Accept-Language", "en-US")
                .header("Referer", "https://www.youtube.com/")
                .send()
                .await?
                .text()
                .await?;
            let c = || -> Result<serde_json::Value, Box<dyn std::error::Error>> {
                let re = Regex::new(r"ytInitialPlayerResponse\s*=\s*(\{.+?\});.*?</script>").unwrap();
                let j: serde_json::Value = serde_json::from_str(re.captures(&resp).ok_or("get_live err 4")?[1].to_string().as_ref())?;
                if j.pointer("/videoDetails/isLive").ok_or("get_live err 7")?.as_bool().ok_or("get_live err 7-2")? == false {
                    return Err(Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "no stream",
                    )));
                }
                Ok(j)
            };
            let j = if let Ok(it) = c() {
                it
            } else {
                if cid.is_empty() {
                    break;
                }
                let ch_url = format!("https://www.youtube.com/channel/{}/videos", &cid);
                let resp = client
                    .get(&ch_url)
                    .header("User-Agent", crate::utils::gen_ua())
                    .header("Accept-Language", "en-US")
                    .header("Referer", "https://www.youtube.com/")
                    .send()
                    .await?
                    .text()
                    .await?;
                let re = fancy_regex::Regex::new(r#""gridVideoRenderer"((.(?!"gridVideoRenderer"))(?!"style":"UPCOMING"))+"label":"(LIVE|LIVE NOW|PREMIERING NOW)"([\s\S](?!"style":"UPCOMING"))+?("gridVideoRenderer"|</script>)"#).unwrap();
                let t = re.captures(&resp)?.ok_or("get_live err 2")?.get(0).ok_or("get_live err 2 2")?.as_str();
                let re = Regex::new(r#""gridVideoRenderer".+?"videoId":"(.+?)""#).unwrap();
                let vid = re.captures(t).ok_or("get_live err 3")?[1].to_string();
                vurl.clear();
                vurl.push_str(format!("https://www.youtube.com/watch?v={}", &vid).as_str());
                continue;
            };
            dash_urls.clear();
            let mut url_v = "".to_owned();
            let mut url_a = "".to_owned();
            for u in j.pointer("/streamingData/adaptiveFormats").ok_or("get_live err 5")?.as_array().ok_or("get_live err 5-2")? {
                if url_v.is_empty() {
                    if u.pointer("/mimeType").ok_or("get_live err 5-3")?.as_str().ok_or("get_live err 5-4")?.starts_with("video") {
                        if let Some(j_u) = u.pointer("/url") {
                            url_v.push_str(j_u.as_str().ok_or("get_live err 5-6")?);
                        } else {
                            let re = Regex::new(r"url=([^&]+)").unwrap();
                            let temp_url = urlencoding::decode(
                                re.captures(u.pointer("/signatureCipher").ok_or("get_live err 5-5")?.as_str().ok_or("get_live err 5-12")?)
                                    .ok_or("get_live err 5-11")?[1]
                                    .to_string()
                                    .as_str(),
                            )?
                            .to_string();
                            let resp =
                                client.get(temp_url).header("User-Agent", crate::utils::gen_ua()).send().await?.json::<serde_json::Value>().await?;
                            // let resp = client.get(temp_url).header("User-Agent", crate::utils::gen_ua()).send().await?.text().await?;
                            // println!("{}", resp);
                            url_v.push_str(resp.pointer("/url").ok_or("get_live err 5-15")?.as_str().ok_or("get_live err 5-16")?);
                        }
                    }
                }
                if url_a.is_empty() {
                    if u.pointer("/mimeType").ok_or("get_live err 5-7")?.as_str().ok_or("get_live err 5-8")?.starts_with("audio") {
                        if let Some(j_u) = u.pointer("/url") {
                            url_a.push_str(j_u.as_str().ok_or("get_live err 5-10")?);
                        } else {
                            let re = Regex::new(r"url=([^&]+)").unwrap();
                            let temp_url = urlencoding::decode(
                                re.captures(u.pointer("/signatureCipher").ok_or("get_live err 5-9")?.as_str().ok_or("get_live err 5-13")?)
                                    .ok_or("get_live err 5-14")?[1]
                                    .to_string()
                                    .as_str(),
                            )?
                            .to_string();
                            let resp =
                                client.get(temp_url).header("User-Agent", crate::utils::gen_ua()).send().await?.json::<serde_json::Value>().await?;
                            url_a.push_str(resp.pointer("/url").ok_or("get_live err 5-17")?.as_str().ok_or("get_live err 5-18")?);
                        }
                    }
                }
            }
            if !url_v.is_empty() && !url_a.is_empty() {
                dash_urls.push_str(&url_v);
                dash_urls.push('\n');
                dash_urls.push_str(&url_a);
                let resp = client.get(&url_v).header("User-Agent", crate::utils::gen_ua()).header("Range", "bytes=0-1").send().await?;
                // println!("{:?}", resp.text().await);
                dash_urls.push('\n');
                dash_urls.push_str(resp.headers().get("X-Head-Seqnum").ok_or("get_live err 9")?.to_str()?);
            }
            ret.insert(
                String::from("title"),
                j.pointer("/videoDetails/title").ok_or("get_live err 8")?.as_str().ok_or("get_live err 8-2")?.to_owned(),
            );
            break;
        }
        if dash_urls.is_empty() {
            return Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "no stream",
            )));
        }
        ret.insert(String::from("url"), dash_urls);
        Ok(ret)
    }
}
