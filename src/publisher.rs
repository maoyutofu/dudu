use log::info;
use rusty_ffmpeg::ffi::{
    av_dict_set, av_dump_format, av_err2str, av_find_best_stream, av_gettime,
    av_interleaved_write_frame, av_packet_unref, av_read_frame, av_rescale_q, av_rescale_q_rnd,
    av_usleep, av_write_trailer, avcodec_parameters_copy, avformat_alloc_output_context2,
    avformat_close_input, avformat_find_stream_info, avformat_free_context, avformat_new_stream,
    avformat_open_input, avformat_write_header, avio_closep, avio_open, AVCodecParameters,
    AVDictionary, AVFormatContext, AVMediaType_AVMEDIA_TYPE_AUDIO as AVMEDIA_TYPE_AUDIO,
    AVMediaType_AVMEDIA_TYPE_SUBTITLE as AVMEDIA_TYPE_SUBTITLE,
    AVMediaType_AVMEDIA_TYPE_VIDEO as AVMEDIA_TYPE_VIDEO, AVOutputFormat, AVPacket, AVRational,
    AVRounding_AV_ROUND_PASS_MINMAX as AV_ROUND_PASS_MINMAX, AVStream, AVERROR_EOF,
    AVERROR_UNKNOWN, AVFMT_NOFILE, AVIO_FLAG_WRITE, AV_NOPTS_VALUE, AV_TIME_BASE,
};
use std::convert::TryInto;
use std::ffi::CString;
use std::sync::atomic::AtomicI32;
use std::sync::atomic::Ordering::*;

fn c_str(s: &str) -> CString {
    CString::new(s).expect("str to c str")
}

pub struct Publisher {
    pub id: i32,
    pub exit_code: AtomicI32,
}

impl Publisher {
    pub fn new(id: i32) -> Self {
        Publisher {
            id: id,
            exit_code: AtomicI32::new(0),
        }
    }
    pub fn av_dict_set(
        &self,
        opts: *mut *mut AVDictionary,
        key: &str,
        value: &str,
        flags: i32,
    ) -> i32 {
        unsafe { av_dict_set(opts, c_str(key).as_ptr(), c_str(value).as_ptr(), flags) as i32 }
    }

    pub async fn start(&self, in_file: &str, out_file: &str) -> Result<(), String> {
        unsafe {
            let mut ofmt_ptr:*const AVOutputFormat = std::ptr::null_mut();
            let mut ifmt_ctx_ptr: *mut AVFormatContext = std::ptr::null_mut();
            let mut ofmt_ctx_ptr: *mut AVFormatContext = std::ptr::null_mut();
            let mut pkt: AVPacket = std::mem::zeroed();
            let mut ret;
            let is_tcp = true;
            let in_filename = c_str(in_file);
            let out_filename = c_str(out_file);
            let format = c_str("flv");
            let mut opts: *mut AVDictionary = std::ptr::null_mut();
            // 设置缓存大小，1080p可将值调大
            self.av_dict_set(&mut opts, "buffer_size", "1024000", 0);
            // 采集buffer
            self.av_dict_set(&mut opts, "rtbufsize", "10000", 0);
            // 设置超时3秒 设置超时断开连接时间，单位微秒
            self.av_dict_set(&mut opts, "stimeout", "3000000", 0);
            // 设置最大时延
            self.av_dict_set(&mut opts, "max_delay", "5000000", 0);
            // 以udp方式打开，如果以tcp方式打开将udp替换为tcp
            // transport tcp
            if is_tcp {
                self.av_dict_set(&mut opts, "rtsp_transport", "tcp", 0);
            }
            'outer: loop {
                // 打开视频输入
                ret = avformat_open_input(
                    &mut ifmt_ctx_ptr,
                    in_filename.as_ptr(),
                    std::ptr::null_mut(),
                    &mut opts,
                );
                if ret < 0 {
                    info!("Could not open input file {:?}", in_filename);
                    break 'outer;
                }
                // 读取视频输入信息
                ret = avformat_find_stream_info(ifmt_ctx_ptr, std::ptr::null_mut());
                if ret < 0 {
                    info!("Failed to retrieve input stream information");
                    break 'outer;
                }
                av_dump_format(ifmt_ctx_ptr, 0, in_filename.as_ptr(), 0);
                let video_index = av_find_best_stream(
                    ifmt_ctx_ptr,
                    AVMEDIA_TYPE_VIDEO,
                    -1,
                    -1,
                    std::ptr::null_mut(),
                    0,
                );
                // 初始化输出封装器
                avformat_alloc_output_context2(
                    &mut ofmt_ctx_ptr,
                    std::ptr::null_mut(),
                    format.as_ptr(),
                    out_filename.as_ptr(),
                );
                if ofmt_ctx_ptr.is_null() {
                    info!("Could not create output context");
                    ret = AVERROR_UNKNOWN;
                    break 'outer;
                }
                let ifmt_ctx = &mut *ifmt_ctx_ptr;
                let ofmt_ctx = &mut *ofmt_ctx_ptr;
                let in_nb_streams = ifmt_ctx.nb_streams as usize;
                let in_streams: &[*mut AVStream] =
                    std::slice::from_raw_parts(ifmt_ctx.streams, in_nb_streams);
                let mut stream_index = 0;
                let mut stream_mapping: Vec<i32> = Vec::with_capacity(in_nb_streams);
                stream_mapping.resize(stream_mapping.capacity(), -1);
                ofmt_ptr = ofmt_ctx.oformat;
                for i in 0..in_nb_streams {
                    let in_stream = &mut *in_streams[i];
                    let in_codecpar_ptr: *mut AVCodecParameters = in_stream.codecpar;
                    let in_codecpar = &mut *in_codecpar_ptr;
                    if in_codecpar.codec_type != AVMEDIA_TYPE_AUDIO
                        && in_codecpar.codec_type != AVMEDIA_TYPE_VIDEO
                        && in_codecpar.codec_type != AVMEDIA_TYPE_SUBTITLE
                    {
                        stream_mapping[i] = -1;
                        continue;
                    }
                    stream_mapping[i] = stream_index;
                    stream_index += 1;
                    // 添加视频流
                    let out_stream_ptr = avformat_new_stream(ofmt_ctx_ptr, std::ptr::null_mut());
                    if out_stream_ptr.is_null() {
                        info!("Failed allocating output stream");
                        ret = AVERROR_UNKNOWN;
                        break 'outer;
                    }
                    let out_stream = &mut *out_stream_ptr;
                    // 复制参数
                    ret = avcodec_parameters_copy(out_stream.codecpar, in_codecpar_ptr);
                    if ret < 0 {
                        info!("Failed to copy codec parameters");
                        break 'outer;
                    }
                    (*out_stream.codecpar).codec_tag = 0;
                }
                av_dump_format(ofmt_ctx_ptr, 0, out_filename.as_ptr(), 1);
                if ((*ofmt_ptr).flags & AVFMT_NOFILE as i32) != AVFMT_NOFILE as i32 {
                    // 打开rtmp网络流
                    ret = avio_open(
                        &mut ofmt_ctx.pb,
                        out_filename.as_ptr(),
                        AVIO_FLAG_WRITE as i32,
                    );
                    if ret < 0 {
                        info!("Could not open output file {:?}", out_filename);
                        break 'outer;
                    }
                }
                // 写入封装头
                ret = avformat_write_header(ofmt_ctx_ptr, std::ptr::null_mut());
                if ret < 0 {
                    info!("Error occurred when opening output file");
                    break 'outer;
                }
                let out_streams = std::slice::from_raw_parts(
                    ofmt_ctx.streams,
                    ofmt_ctx.nb_streams.try_into().unwrap(),
                );
                let mut cur_pts: [i64; 64] = [0; 64];
                let start_time = av_gettime();
                'inner: loop {
                    let exit_code = self.exit_code.load(SeqCst);
                    if exit_code == 1 {
                        info!("stopped.");
                        break;
                    }
                    // 获取摄像头帧
                    ret = av_read_frame(ifmt_ctx_ptr, &mut pkt);
                    if ret < 0 {
                        break 'inner;
                    }
                    let curr_stream_index = pkt.stream_index as usize;
                    let in_stream_ptr = in_streams[curr_stream_index];
                    if curr_stream_index >= stream_mapping.len()
                        || stream_mapping[curr_stream_index] < 0
                    {
                        av_packet_unref(&mut pkt);
                        continue;
                    }
                    pkt.stream_index = stream_mapping[curr_stream_index];
                    let out_stream_ptr = out_streams[curr_stream_index];
                    let in_stream = &mut *in_stream_ptr;
                    let out_stream = &mut *out_stream_ptr;
                    let orig_pts = pkt.pts;
                    let orig_duration = pkt.duration;
                    if orig_pts == AV_NOPTS_VALUE {
                        pkt.pts = cur_pts[curr_stream_index];
                        pkt.dts = pkt.pts;
                    }
                    if pkt.stream_index == video_index {
                        let time_base =
                            (*(*ifmt_ctx.streams.offset(video_index as isize))).time_base;
                        let time_base_q = AVRational {
                            num: 1,
                            den: AV_TIME_BASE as i32,
                        };
                        let pts_time = av_rescale_q(pkt.dts, time_base, time_base_q);
                        let now_time = av_gettime() - start_time;
                        if pts_time > now_time {
                            av_usleep((pts_time - now_time) as u32);
                        }
                    }
                    // log_packet(ifmt_ctx_ptr, &pkt, "in");
                    /* copy packet */
                    pkt.pts = av_rescale_q_rnd(
                        pkt.pts,
                        in_stream.time_base,
                        out_stream.time_base,
                        AV_ROUND_PASS_MINMAX,
                    );
                    pkt.dts = av_rescale_q_rnd(
                        pkt.dts,
                        in_stream.time_base,
                        out_stream.time_base,
                        AV_ROUND_PASS_MINMAX,
                    );
                    pkt.duration =
                        av_rescale_q(pkt.duration, in_stream.time_base, out_stream.time_base);
                    pkt.pos = -1;
                    // log_packet(ofmt_ctx_ptr, &pkt, "out");
                    // 发送到服务器
                    ret = av_interleaved_write_frame(ofmt_ctx_ptr, &mut pkt);
                    if ret < 0 {
                        info!("Error muxing packet");
                        break 'inner;
                    }
                    if orig_pts == AV_NOPTS_VALUE {
                        cur_pts[curr_stream_index] += orig_duration;
                    }
                    av_packet_unref(&mut pkt);
                }
                av_write_trailer(ofmt_ctx_ptr);
                break 'outer;
            }
            avformat_close_input(&mut ifmt_ctx_ptr);
            // close output
            if !ofmt_ctx_ptr.is_null()
                && ((*ofmt_ptr).flags & AVFMT_NOFILE as i32) != AVFMT_NOFILE as i32
            {
                avio_closep(&mut (*ofmt_ctx_ptr).pb);
            }
            avformat_free_context(ofmt_ctx_ptr);
            if ret < 0 && ret != AVERROR_EOF {
                info!("Error occurred: {:?}", av_err2str(ret));
                // std::process::exit(-2);
                return Err(av_err2str(ret));
            }
        }
        Ok(())
    }

    pub fn stop(&self) -> bool {
        info!("stoping...");
        self.exit_code.store(1, SeqCst);
        true
    }
}
